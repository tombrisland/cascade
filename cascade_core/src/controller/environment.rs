use crate::connection::{ComponentChannels, Connection};
use async_channel::SendError;
use async_trait::async_trait;
use cascade_api::component::environment::ComponentEnvironment;
use cascade_api::component::error::ComponentError;
use cascade_api::connection::definition::DEFAULT_CONNECTION;
use cascade_api::message::Message;
use futures::stream::{select_all, SelectAll};
use futures_util::StreamExt;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

/// Wraps async-channel receivers to create a fused stream
///
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
    // Connections which can be ignored if they don't exist
    ignore_connections: Vec<String>,

    // The message in process combined with the connection it came from
    in_progress: Option<(String, Message)>,

    rx: FusedConnections,
    pub shutdown_token: CancellationToken,
    tx_named: HashMap<String, Connection>,
}

#[async_trait]
impl ComponentEnvironment for ExecutionEnvironment {
    async fn recv<'a>(&'a mut self) -> Result<&'a Message, ComponentError> {
        let next: Result<Option<(String, Message)>, ComponentError> = tokio::select! {
            // Ensure we always check shutdown token first
            biased;

            _ = self.shutdown_token.cancelled() => { Err(ComponentError::ComponentShutdown) },
            message = self.rx.recv() => { Ok(message) }
        };

        // Store item in progress in this session
        let (_, message): &mut (String, Message) = self
            .in_progress
            // The stream returns None when all inputs are closed
            .insert(next?.ok_or(ComponentError::InputClosed)?);

        Ok(message)
    }

    async fn send(&mut self, name: &str, item: Message) -> Result<(), ComponentError> {
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

    async fn send_default(&mut self, item: Message) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}

impl ExecutionEnvironment {
    pub fn new(
        channels: ComponentChannels,
        shutdown_token: CancellationToken,
    ) -> ExecutionEnvironment {
        ExecutionEnvironment {
            ignore_connections: vec![DEFAULT_CONNECTION.to_string()],
            in_progress: None,
            rx: FusedConnections::new(channels.rx),
            shutdown_token,
            tx_named: channels.tx_named,
        }
    }

    /// Ensure all output channels have capacity
    pub fn tx_has_capacity(&self) -> bool {
        self.tx_named
            .iter()
            .all(|(_, conn)| conn.tx.len() < conn.metadata.capacity)
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
