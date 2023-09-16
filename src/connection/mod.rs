use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::Mutex;

use crate::graph::item::CascadeItem;

#[derive(Clone, Deserialize)]
pub struct ConnectionDefinition {
    pub from: usize,
    pub to: usize,

    pub max_items: u32,
}

pub struct Connection {
    item_count: AtomicI32,
    max_items: u32,

    // TODO no need to be mutex
    tx: Mutex<Sender<CascadeItem>>,
    rx: Mutex<Receiver<CascadeItem>>,
}

impl Connection {
    pub fn new(def: ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) = channel(def.max_items as usize);

        Connection { item_count: Default::default(), max_items: def.max_items, tx: Mutex::new(tx), rx: Mutex::new(rx) }
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

#[derive(Clone)]
pub struct Connections {
    pub connections: Vec<Arc<Connection>>,
}

impl Connections {
    pub fn new(connections: Vec<Arc<Connection>>) -> Connections {
        Connections {
            connections
        }
    }

    pub async fn send(&self, value: CascadeItem) -> Result<(), SendError<CascadeItem>> {
        for connection in &self.connections {
            connection.send(value.clone()).await?
        }

        Ok(())
    }
}