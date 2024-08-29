use std::collections::HashMap;

use async_channel::{bounded, Receiver, Sender};

use definition::ConnectionDefinition;
use crate::message::Message;

pub mod definition;

#[derive(Clone)]
pub struct Connection {
    pub name: String,
    pub capacity: usize,

    pub rx: Receiver<Message>,
    pub tx: Sender<Message>,
}

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (
            Sender<Message>,
            Receiver<Message>,
        ) = bounded(def.capacity);

        Connection {
            name: def.name.clone(),
            capacity: def.capacity,
            rx,
            tx,
        }
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Receiver<Message>>,

    // Named output connections
    pub tx_named: HashMap<String, Sender<Message>>,
}
