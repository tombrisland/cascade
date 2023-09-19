use std::sync::Arc;

use futures::future::join_all;
use futures::stream::{select_all, SelectAll};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

use definition::ConnectionDefinition;

use crate::graph::item::CascadeItem;

pub mod definition;

pub struct Connection {
    pub name: String,

    pub rx: Arc<RwLock<Option<Receiver<CascadeItem>>>>,
    pub tx: Arc<Sender<CascadeItem>>,
}

pub type IncomingStream = SelectAll<ReceiverStream<CascadeItem>>;

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) = channel(def.max_items as usize);

        Connection {
            name: def.name.clone(),
            rx: Arc::new(RwLock::new(Some(rx))),
            tx: Arc::new(tx),
        }
    }

    pub async fn get_receiver_stream(&self) -> ReceiverStream<CascadeItem> {
        self.rx.clone().write_owned().await.take().map(ReceiverStream::new).unwrap()
    }
}