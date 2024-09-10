use crate::component::component::ComponentMetadata;
use crate::component::error::ComponentError;
use crate::connection::definition::DEFAULT_CONNECTION;
use crate::connection::{ComponentChannels, Connection};
use crate::message::Message;
use async_channel::SendError;
use futures::future::{select, Either};
use futures::stream::{select_all, SelectAll};
use futures::StreamExt;
use std::collections::HashMap;
use std::pin::pin;
use tokio_util::sync::CancellationToken;

/// Wraps async-channel receivers to create a fused stream
/// Multiple input streams can then be read from the same stream
pub struct FusedConnections {
    select_all: SelectAll<Connection>,
}

impl FusedConnections {
    pub fn new(receivers: Vec<Connection>) -> FusedConnections {
        FusedConnections {
            select_all: select_all(receivers),
        }
    }

    pub(crate) async fn recv(&mut self) -> Option<(String, Message)> {
        self.select_all.next().await
    }
}

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Connections which can be ignored if they don't exist
    ignore_connections: Vec<String>,

    // The message in process combined with the connection it came from
    in_progress: Option<(String, Message)>,

    rx: FusedConnections,
    pub shutdown_token: CancellationToken,
    tx_named: HashMap<String, Connection>,
}

impl ExecutionEnvironment {
    pub fn new(
        metadata: ComponentMetadata,
        channels: ComponentChannels,
        shutdown_token: CancellationToken,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            metadata,
            ignore_connections: vec![DEFAULT_CONNECTION.to_string()],
            in_progress: None,
            rx: FusedConnections::new(channels.rx),
            shutdown_token,
            tx_named: channels.tx_named,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<&Message, ComponentError> {
        // TODO probably want to bias to shutdown path
        match select(pin!(self.rx.recv()), pin!(self.shutdown_token.cancelled())).await {
            Either::Left((message, _)) => {
                // Store item in progress in this session
                let (_, message): &mut (String, Message) = self
                    .in_progress
                    .insert(message.ok_or(ComponentError::InputClosed)?);

                Ok(message)
            }
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

    pub async fn complete(&mut self) {
        self.in_progress.take();
    }

    /// Return the in-flight item to its original queue
    pub async fn rollback(&mut self) -> Result<(), SendError<Message>> {
        if let Some((conn_id, message)) = self.in_progress.take() {
            let option: Option<&Connection> = self
                .rx
                .select_all
                .iter()
                .find(|conn| conn.metadata.id == conn_id);

            if let Some(conn) = option {
                conn.send(message).await?
            }
        }

        Ok(())
    }
}
