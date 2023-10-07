use std::collections::HashMap;

use async_channel::{Receiver, Sender};
use futures::stream::{select_all, SelectAll};
use futures::StreamExt;

use crate::component::component::ComponentMetadata;
use crate::component::error::ComponentError;
use crate::connection::ComponentChannels;
use crate::connection::definition::DEFAULT_CONNECTION;
use crate::message::{InternalMessage, Message};

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

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Connections which can be ignored if they don't exist
    ignore_connections: Vec<String>,

    in_progress: Option<Message>,

    rx: FusedStream<InternalMessage>,
    tx_named: HashMap<String, Sender<InternalMessage>>,
}

impl ExecutionEnvironment {
    pub fn new(metadata: ComponentMetadata, channels: ComponentChannels) -> ExecutionEnvironment {
        ExecutionEnvironment {
            metadata,
            ignore_connections: vec![DEFAULT_CONNECTION.to_string()],
            in_progress: None,
            rx: FusedStream::new(channels.rx),
            tx_named: channels.tx_named,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<&Message, ComponentError> {
        match self.rx.recv().await.ok_or(ComponentError::InputClosed)? {
            InternalMessage::Item(item) => {
                // Store the item in-progress
                Ok(self.in_progress.insert(item))
            }
            InternalMessage::ShutdownSignal => Err(ComponentError::ComponentShutdown),
        }
    }

    pub async fn send(&mut self, name: &str, item: Message) -> Result<(), ComponentError> {
        match self.tx_named.get(name) {
            Some(connection) => {
                connection
                    .send(InternalMessage::Item(item))
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
