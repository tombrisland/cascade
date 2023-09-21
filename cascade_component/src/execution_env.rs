use std::collections::HashMap;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use futures::stream::{select_all, SelectAll};
use futures::StreamExt;

use cascade_connection::definition::DEFAULT_CONNECTION;
use cascade_connection::{ComponentChannels, Message};
use cascade_payload::CascadeItem;

use crate::component::ComponentMetadata;
use crate::error::ComponentError;

type FusedStream<T> = SelectAll<Receiver<T>>;
pub struct ReceiverStream<Message> {
    select_all: FusedStream<Message>,
}

impl<Message> ReceiverStream<Message> {
    fn new(receivers: Vec<Receiver<Message>>) -> ReceiverStream<Message> {
        ReceiverStream {
            select_all: select_all(receivers),
        }
    }

    async fn recv(&mut self) -> Option<Message> {
        self.select_all.next().await
    }
}

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Connections which can be ignored if they don't exist
    ignore_connections: Vec<String>,

    pub rx: ReceiverStream<Message>,
    tx_named: HashMap<String, Arc<Sender<Message>>>,
}

impl ExecutionEnvironment {
    pub fn new(metadata: ComponentMetadata, channels: ComponentChannels) -> ExecutionEnvironment {
        ExecutionEnvironment {
            metadata,
            ignore_connections: vec![DEFAULT_CONNECTION.to_string()],
            rx: ReceiverStream::new(channels.rx),
            tx_named: channels.tx_named,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<CascadeItem, ComponentError> {
        match self.rx.recv().await.ok_or(ComponentError::InputClosed)? {
            Message::Item(item) => Ok(item),
            Message::ShutdownSignal => Err(ComponentError::ComponentShutdown),
        }
    }

    pub async fn send(&self, name: &str, item: CascadeItem) -> Result<(), ComponentError> {
        match self.tx_named.get(name) {
            Some(connection) => connection
                .send(Message::Item(item))
                .await
                .or(Err(ComponentError::OutputClosed)),
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
    pub async fn send_default(&self, item: CascadeItem) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}
