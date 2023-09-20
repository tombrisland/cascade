use std::sync::Arc;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

use cascade_payload::CascadeItem;
use definition::ConnectionDefinition;

pub mod definition;

pub struct Connection {
    pub name: String,
    pub idx: usize,

    pub rx: Arc<RwLock<Option<Receiver<CascadeItem>>>>,
    pub tx: Arc<Sender<CascadeItem>>,
}

impl Connection {
    pub fn new(edge_idx: usize, def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) =
            channel(def.max_items as usize);

        Connection {
            idx: edge_idx,
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