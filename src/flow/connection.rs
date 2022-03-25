use mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc, Mutex};

use crate::flow::item::FlowItem;

#[derive(Clone)]
// Describes the connection between components
pub struct FlowConnection {
    // Send to the connection
    pub tx: Sender<FlowItem>,

    // Receive from the connection
    pub rx: Arc<Mutex<Receiver<FlowItem>>>,
}

// TODO start and stop connections

impl FlowConnection {
    pub fn new() -> FlowConnection {
        let (tx, rx) = mpsc::channel();

        FlowConnection {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}

pub struct ComponentConnections {
    pub incoming: Vec<FlowConnection>,
    pub outgoing: Vec<FlowConnection>,
}

impl ComponentConnections {
    pub fn get_incoming_rx(&self) -> Vec<Arc<Mutex<Receiver<FlowItem>>>> {
        self.incoming.iter().map(|conn| { conn.rx.clone() }).collect()
    }

    pub fn get_incoming_tx(&self) -> Vec<Sender<FlowItem>> {
        self.incoming.iter().map(|conn| { conn.tx.clone() }).collect()
    }

    pub fn get_outgoing_tx(&self) -> Vec<Sender<FlowItem>> {
        self.outgoing.iter().map(|conn| { conn.tx.clone() }).collect()
    }
}