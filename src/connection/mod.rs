use std::ops::DerefMut;
use std::sync::atomic::{AtomicI32, Ordering};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;

use crate::graph::item::CascadeItem;

pub struct ConnectionEdge {
    id: String,

    item_count: AtomicI32,
    max_items: u32,

    tx: Mutex<Sender<CascadeItem>>,
    rx: Mutex<Receiver<CascadeItem>>,
}

impl ConnectionEdge {
    pub fn new(id: String, max_items: usize) -> ConnectionEdge {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) = channel(max_items);

        ConnectionEdge { id, item_count: Default::default(), max_items: max_items.try_into().unwrap(), tx: Mutex::new(tx), rx: Mutex::new(rx) }
    }

    pub async fn send(&self, value: CascadeItem) -> Result<(), SendError<CascadeItem>> {
        let sender: &Sender<CascadeItem> = &*self.tx.lock().await;

        self.item_count.fetch_add(1, Ordering::Relaxed);

        sender.send(value).await
    }

    pub async fn recv(&self) -> Option<CascadeItem> {
        let mut guard = self.rx.lock().await;

        let receiver: &mut Receiver<CascadeItem> = guard.deref_mut();
        self.item_count.fetch_add(-1, Ordering::Relaxed);

        receiver.recv().await
    }
}