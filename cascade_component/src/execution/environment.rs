use std::collections::HashMap;
use std::sync::Arc;

use async_channel::Sender;

use cascade_connection::{ComponentChannels, Message};
use cascade_connection::definition::DEFAULT_CONNECTION;
use cascade_payload::CascadeMessage;

use crate::component::ComponentMetadata;
use crate::error::ComponentError;
use crate::execution::stream::ReceiverStream;

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
    pub async fn recv(&mut self) -> Result<CascadeMessage, ComponentError> {
        match self.rx.recv().await.ok_or(ComponentError::InputClosed)? {
            Message::Item(item) => Ok(item),
            Message::ShutdownSignal => Err(ComponentError::ComponentShutdown),
        }
    }

    pub async fn send(&self, name: &str, item: CascadeMessage) -> Result<(), ComponentError> {
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
    pub async fn send_default(&self, item: CascadeMessage) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}
