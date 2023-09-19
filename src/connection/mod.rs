use std::collections::HashMap;
use std::sync::Arc;

use petgraph::Direction;
use petgraph::graph::EdgeIndex;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

use definition::ConnectionDefinition;

use crate::graph::item::CascadeItem;

pub mod definition;

pub struct Connection {
    pub name: String,
    pub idx: EdgeIndex,

    pub rx: Arc<RwLock<Option<Receiver<CascadeItem>>>>,
    pub tx: Arc<Sender<CascadeItem>>,
}

impl Connection {
    pub fn new(edge_idx: &EdgeIndex, def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) =
            channel(def.max_items as usize);

        Connection {
            idx: edge_idx.clone(),
            name: def.name.clone(),
            rx: Arc::new(RwLock::new(Some(rx))),
            tx: Arc::new(tx),
        }
    }

    pub async fn get_receiver_stream(&self) -> ReceiverStream<CascadeItem> {
        self.rx
            .clone()
            .write_owned()
            .await
            .take()
            .map(ReceiverStream::new)
            .unwrap()
    }
}

pub struct DirectedConnections {
    pub connections: HashMap<Direction, Vec<Arc<Connection>>>,
}

impl DirectedConnections {
    pub fn new(connections_list: Vec<(Direction, Arc<Connection>)>) -> DirectedConnections {
        let mut connections: HashMap<Direction, Vec<Arc<Connection>>> = Default::default();

        for (direction, connection) in connections_list {
            connections
                .entry(direction)
                .or_insert_with(Vec::new)
                .push(connection);
        }

        DirectedConnections { connections }
    }
}
