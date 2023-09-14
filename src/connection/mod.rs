use std::ops::DerefMut;
use std::sync::atomic::{AtomicI32, Ordering};

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;

use crate::flow::item::FlowItem;

pub struct ConnectionEdge {
    id: String,
    
    item_count: AtomicI32,
    max_items: u32,

    tx: Mutex<Sender<FlowItem>>,
    rx: Mutex<Receiver<FlowItem>>,
}

impl ConnectionEdge {
    pub fn new(id: String, max_items: usize) -> ConnectionEdge {
        let (tx, rx): (Sender<FlowItem>, Receiver<FlowItem>) = channel(max_items);

        ConnectionEdge { id, item_count: Default::default(), max_items: max_items.try_into().unwrap(), tx: Mutex::new(tx), rx: Mutex::new(rx) }
    }

    pub async fn send(&self, value: FlowItem) -> Result<(), SendError<FlowItem>> {
        let sender: &Sender<FlowItem> = &*self.tx.lock().await;

        self.item_count.fetch_add(1, Ordering::Relaxed);

        sender.send(value).await
    }

    pub async fn recv(&self) -> Option<FlowItem> {
        let mut guard = self.rx.lock().await;

        let receiver: &mut Receiver<FlowItem> = guard.deref_mut();
        self.item_count.fetch_add(-1, Ordering::Relaxed);

        receiver.recv().await
    }
}