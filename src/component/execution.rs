use std::collections::HashMap;
use std::sync::Arc;

use futures::stream::SelectAll;
use futures::StreamExt;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use crate::component::error::ComponentError;
use crate::connection::ConnectionWrite;
use crate::graph::graph::CascadeGraph;
use crate::graph::item::CascadeItem;

type IncomingStream = SelectAll<ReceiverStream<CascadeItem>>;

pub struct ComponentEnv {
    // Stream from all incoming connections
    incoming_stream: Option<IncomingStream>,

    outgoing_connections: HashMap<String, Arc<ConnectionWrite>>,
}

pub const DEFAULT_CONNECTION: &str = "default";

impl ComponentEnv {
    pub fn new(
        graph: &CascadeGraph,
        incoming_stream: IncomingStream,
        outgoing: Vec<Arc<ConnectionWrite>>,
    ) -> ComponentEnv {
        let outgoing_connections: HashMap<String, Arc<ConnectionWrite>> = outgoing
            .into_iter()
            .map(|conn| (conn.name.clone(), conn))
            .collect();

        ComponentEnv {
            incoming_stream,
            outgoing_connections,
        }
    }

    // Get a single item from the session
    pub async fn recv(&mut self) -> Result<CascadeItem, ComponentError> {
        let stream: &mut IncomingStream = self.incoming_stream.as_mut().ok_or(ComponentError::MissingInputConnection)?;

        stream.next().await.ok_or(ComponentError::InputClosed)
    }

    pub async fn send(&self, name: &str, item: CascadeItem) -> Result<(), ComponentError> {
        match self.outgoing_connections.get(name) {
            Some(connection) => connection
                .tx
                .send(item)
                .await
                .or(Err(ComponentError::OutputClosed)),
            None => {
                // Don't try and send if there are no attached connections
                // TODO decide on missing connection behaviour
                if self.outgoing_connections.is_empty() {
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
