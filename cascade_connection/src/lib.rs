use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_channel::{bounded, Receiver, Sender};

use cascade_payload::CascadeMessage;
use definition::ConnectionDefinition;

pub mod definition;

pub enum Message {
    ShutdownSignal,
    Item(CascadeMessage),
}

pub struct Connection {
    pub name: String,

    pub rx: Arc<RwLock<Option<Receiver<Message>>>>,
    pub tx: Arc<Sender<Message>>,
}

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(def.max_items as usize);

        Connection {
            name: def.name.clone(),
            rx: Arc::new(RwLock::new(Some(rx))),
            tx: Arc::new(tx),
        }
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Receiver<Message>>,
    // Sender to dispatch signals to the component
    pub tx_signal: Arc<Sender<Message>>,

    // Named output connections
    pub tx_named: HashMap<String, Arc<Sender<Message>>>,
}
