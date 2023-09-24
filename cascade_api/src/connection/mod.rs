use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_channel::{bounded, Receiver, Sender};

use definition::ConnectionDefinition;

use crate::message::InternalMessage;

pub mod definition;

pub struct Connection {
    pub name: String,
    pub max_items: usize,

    pub rx: Arc<RwLock<Option<Receiver<InternalMessage>>>>,
    pub tx: Arc<Sender<InternalMessage>>,
}

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<InternalMessage>, Receiver<InternalMessage>) = bounded(def.max_items);

        Connection {
            name: def.name.clone(),
            max_items: def.max_items,
            rx: Arc::new(RwLock::new(Some(rx))),
            tx: Arc::new(tx),
        }
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Receiver<InternalMessage>>,
    // Sender to dispatch signals to the component
    pub tx_signal: Arc<Sender<InternalMessage>>,

    // Named output connections
    pub tx_named: HashMap<String, Arc<Sender<InternalMessage>>>,
}
