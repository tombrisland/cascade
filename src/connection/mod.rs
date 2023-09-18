use futures::stream::{select_all, SelectAll};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::OwnedRwLockWriteGuard;
use tokio_stream::wrappers::ReceiverStream;

use definition::ConnectionDefinition;

use crate::graph::item::CascadeItem;

pub mod definition;

pub struct ConnectionRead {
    pub name: String,

    rx: Option<Receiver<CascadeItem>>,
}

pub struct ConnectionWrite {
    pub name: String,

    pub tx: Sender<CascadeItem>,
}

pub fn create_connection(def: &ConnectionDefinition) -> (ConnectionRead, ConnectionWrite) {
    let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) = channel(def.max_items as usize);

    (
        ConnectionRead {
            name: def.name.clone(),
            rx: Some(rx),
        },
        ConnectionWrite {
            name: def.name.clone(),
            tx,
        },
    )
}

pub type IncomingStream = SelectAll<ReceiverStream<CascadeItem>>;

impl ConnectionRead {
    pub fn get_receiver_stream(&mut self) -> ReceiverStream<CascadeItem> {
        self.rx.take().map(ReceiverStream::new).unwrap()
    }

    pub async fn create_incoming_stream(
        mut incoming: Vec<OwnedRwLockWriteGuard<ConnectionRead>>,
    ) -> IncomingStream {
        let streams: Vec<ReceiverStream<CascadeItem>> = incoming
            .iter_mut()
            .map(|conn| conn.get_receiver_stream())
            .collect();

        select_all(streams)
    }
}
