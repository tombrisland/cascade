use std::collections::HashMap;

use async_channel::{bounded, Receiver, Sender};

use definition::ConnectionDefinition;

use crate::message::InternalMessage;

pub mod definition;

#[derive(Clone)]
pub struct Connection {
    pub name: String,
    pub max_items: usize,

    pub rx: Receiver<InternalMessage>,
    pub tx: Sender<InternalMessage>,
}

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<InternalMessage>, Receiver<InternalMessage>) = bounded(def.max_items);

        Connection {
            name: def.name.clone(),
            max_items: def.max_items,
            rx,
            tx
        }
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Receiver<InternalMessage>>,
    // Sender to dispatch signals to the component
    pub tx_signal: Sender<InternalMessage>,

    // Named output connections
    pub tx_named: HashMap<String, Sender<InternalMessage>>,
}
