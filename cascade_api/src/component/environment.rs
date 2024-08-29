use crate::component::component::ComponentMetadata;
use crate::component::error::ComponentError;
use crate::connection::definition::DEFAULT_CONNECTION;
use crate::connection::ComponentChannels;
use crate::message::Message;
use async_channel::{Receiver, Sender};
use futures::future::{select, Either};
use futures::stream::{select_all, SelectAll};
use futures::StreamExt;
use std::collections::HashMap;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;

/// Wraps async-channel receivers to create a fused stream
/// Multiple input streams can then be read from the same stream
pub struct FusedStream<Message> {
    select_all: SelectAll<Receiver<Message>>,
}

impl<Message> FusedStream<Message> {
    pub fn new(receivers: Vec<Receiver<Message>>) -> FusedStream<Message> {
        FusedStream {
            select_all: select_all(receivers),
        }
    }

    pub(crate) async fn recv(&mut self) -> Option<Message> {
        self.select_all.next().await
    }
}

#[derive(Default)]
pub struct StopComponent {
    is_shutdown: AtomicBool,
    notify: Notify,
}

impl StopComponent {
    pub fn stop(&self, task_count: usize) {
        self.is_shutdown.store(true, Ordering::Relaxed);

        for _ in 0..task_count {
            self.notify.notify_one();
        }
    }

    pub fn wait(&self) -> Notified {
        self.notify.notified()
    }

    pub fn is_stopped(&self) -> bool {
        self.is_shutdown.load(Ordering::Relaxed)
    }
}

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Connections which can be ignored if they don't exist
    ignore_connections: Vec<String>,

    in_progress: Option<Message>,

    rx: FusedStream<Message>,
    pub shutdown: Arc<StopComponent>,
    tx_named: HashMap<String, Sender<Message>>,
}

impl ExecutionEnvironment {
    pub fn new(
        metadata: ComponentMetadata,
        channels: ComponentChannels,
        shutdown: Arc<StopComponent>,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            metadata,
            ignore_connections: vec![DEFAULT_CONNECTION.to_string()],
            in_progress: None,
            rx: FusedStream::new(channels.rx),
            shutdown,
            tx_named: channels.tx_named,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<&Message, ComponentError> {
        match select(pin!(self.rx.recv()), pin!(self.shutdown.wait())).await {
            Either::Left((message, _)) => Ok(self
                .in_progress
                .insert(message.ok_or(ComponentError::InputClosed)?)),
            Either::Right(_) => Err(ComponentError::ComponentShutdown),
        }
    }

    pub async fn send(&mut self, name: &str, item: Message) -> Result<(), ComponentError> {
        match self.tx_named.get(name) {
            Some(connection) => {
                connection
                    .send(item)
                    .await
                    .or(Err(ComponentError::OutputClosed))?;

                // Remove the item from in-progress
                self.in_progress.take();

                Ok(())
            }
            None => {
                // Only error if there are connections that aren't configured to drop
                if self.tx_named.is_empty() || self.ignore_connections.contains(&name.to_string()) {
                    Ok(())
                } else {
                    Err(ComponentError::MissingOutput(name.to_string()))
                }
            }
        }
    }

    // Send to the default connection
    pub async fn send_default(&mut self, item: Message) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}
