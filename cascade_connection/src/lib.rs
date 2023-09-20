use std::sync::Arc;

use async_channel::{bounded, Receiver, Sender};

use cascade_payload::CascadeItem;
use definition::ConnectionDefinition;

pub mod definition;

pub struct Connection {
    pub idx: usize,
    pub name: String,

    pub rx: Arc<Receiver<CascadeItem>>,
    pub tx: Arc<Sender<CascadeItem>>,
}

impl Connection {
    pub fn new(idx: usize, def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<CascadeItem>, Receiver<CascadeItem>) = bounded(def.max_items as usize);

        Connection {
            idx,
            name: def.name.clone(),
            rx: Arc::new(rx),
            tx: Arc::new(tx),
        }
    }
}