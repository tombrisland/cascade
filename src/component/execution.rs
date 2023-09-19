use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use futures::stream::{select_all, SelectAll};
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::component::error::ComponentError;
use crate::connection::Connection;
use crate::graph::item::CascadeItem;

type IncomingStream = SelectAll<ReceiverStream<CascadeItem>>;

pub struct ExecutionEnvironment {
    // Stream from all incoming connections
    incoming_stream: Option<IncomingStream>,

    tx: HashMap<String, Arc<Sender<CascadeItem>>>,
}

pub const DEFAULT_CONNECTION: &str = "default";

impl ExecutionEnvironment {
    pub async fn new(
        connections_incoming: Vec<&Connection>,
        connections_outgoing: Vec<&Connection>,
    ) -> ExecutionEnvironment {
        let incoming_streams: Vec<ReceiverStream<CascadeItem>> = join_all(
            connections_incoming
                .into_iter()
                .map(|conn| conn.get_receiver_stream()),
        )
        .await;

        let incoming_stream: IncomingStream = select_all(incoming_streams);

        let outgoing_connections: HashMap<String, Arc<Sender<CascadeItem>>> = connections_outgoing
            .iter()
            .map(|conn| (conn.name.clone(), Arc::clone(&conn.tx)))
            .collect();

        ExecutionEnvironment {
            incoming_stream: Some(incoming_stream),
            tx: outgoing_connections,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<CascadeItem, ComponentError> {
        let stream: &mut IncomingStream = self
            .incoming_stream
            .as_mut()
            .ok_or(ComponentError::MissingInputConnection)?;

        stream.next().await.ok_or(ComponentError::InputClosed)
    }

    pub async fn send(&self, name: &str, item: CascadeItem) -> Result<(), ComponentError> {
        match self.tx.get(name) {
            Some(connection) => connection
                .send(item)
                .await
                .or(Err(ComponentError::OutputClosed)),
            None => {
                // Don't try and send if there are no attached connections
                // TODO decide on missing connection behaviour
                if self.tx.is_empty() {
                    Ok(())
                } else {
                    Err(ComponentError::MissingOutputConnection(name.to_string()))
                }
            }
        }
    }

    // Send to the default connection
    pub async fn send_default(&self, item: CascadeItem) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}
