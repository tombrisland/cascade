use mpsc::{Receiver, Sender};
use std::sync::{Arc, mpsc, Mutex};

use crate::Item;

// Describes the connection between components
#[derive(Clone)]
pub struct Connection {
    pub tx: Sender<Item>,
    pub rx: Arc<Mutex<Receiver<Item>>>,
}

impl Connection {
    pub fn new() -> Connection {
        let (tx, rx) = mpsc::channel();

        Connection {
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }
}