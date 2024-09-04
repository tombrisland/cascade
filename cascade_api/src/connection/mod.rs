use crate::message::Message;
use async_channel::{bounded, Receiver, Sender};
use definition::ConnectionDefinition;
use futures_core::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::StreamExt;

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
        let (tx, rx): (Sender<Message>, Receiver<Message>) = bounded(def.capacity);

        Connection {
            name: def.name.clone(),
            capacity: def.capacity,
            rx,
            tx,
        }
    }

    /// Wrap send function for the underlying Sender
    pub(crate) fn send(&self, msg: Message) -> async_channel::Send<'_, Message> {
        self.tx.send(msg)
    }
}

impl Stream for Connection {
    type Item = (String, Message);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx
            .poll_next_unpin(cx)
            .map(|opt| opt.map(|msg| (self.name.clone(), msg)))
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Connection>,

    // Named output connections
    pub tx_named: HashMap<String, Connection>,
}
