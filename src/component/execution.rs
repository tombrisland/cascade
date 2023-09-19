use std::collections::HashMap;
use std::sync::Arc;

use futures::future::join_all;
use futures::stream::{select_all, SelectAll};
use petgraph::{Incoming, Outgoing};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::component::component::ComponentMetadata;
use crate::component::error::ComponentError;
use crate::connection::DirectedConnections;
use crate::graph::item::CascadeItem;

type IncomingStream = SelectAll<ReceiverStream<CascadeItem>>;

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Stream from all incoming connections
    incoming_stream: Option<IncomingStream>,
    // Keep track of the edge idx for each underlying stream
    pub receiver_indexes: Vec<usize>,

    tx: HashMap<String, Arc<Sender<CascadeItem>>>,
}

pub const DEFAULT_CONNECTION: &str = "default";

impl ExecutionEnvironment {
    pub async fn new(
        metadata: ComponentMetadata,
        directed_connections: DirectedConnections,
    ) -> ExecutionEnvironment {
        let mut receiver_indexes: Vec<usize> = vec![];

        let incoming_streams: Vec<ReceiverStream<CascadeItem>> = join_all(
            directed_connections
                .connections
                .get(&Incoming)
                .unwrap_or(&Vec::new())
                .into_iter()
                // Remember the order of these streams
                .inspect(|conn| receiver_indexes.push(conn.idx.index()))
                .map(|conn| conn.get_receiver_stream()),
        )
        .await;

        let incoming_stream: IncomingStream = select_all(incoming_streams);

        let outgoing_connections: HashMap<String, Arc<Sender<CascadeItem>>> = directed_connections
            .connections
            .get(&Outgoing)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|conn| (conn.name.clone(), Arc::clone(&conn.tx)))
            .collect();

        ExecutionEnvironment {
            metadata,
            incoming_stream: Some(incoming_stream),
            receiver_indexes,
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

    pub fn take_receivers(&mut self) -> Option<Vec<Receiver<CascadeItem>>> {
        self.incoming_stream.take().map(|stream| {
            stream
                .into_iter()
                .map(|stream| stream.into_inner())
                .collect()
        })
    }
}
