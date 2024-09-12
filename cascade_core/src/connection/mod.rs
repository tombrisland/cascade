use async_channel::{unbounded, Receiver, Sender};
use cascade_api::connection::definition::ConnectionDefinition;
use cascade_api::connection::ConnectionMetadata;
use cascade_api::message::Message;
use futures::StreamExt;
use futures_core::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct Connection {
    pub metadata: ConnectionMetadata,

    pub rx: Receiver<Message>,
    pub tx: Sender<Message>,
}

impl Connection {
    pub fn new(def: &ConnectionDefinition) -> Connection {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = unbounded();

        Connection {
            metadata: ConnectionMetadata {
                id: def.id.clone(),
                name: def.name.clone(),
                capacity: def.capacity,
            },
            rx,
            tx,
        }
    }

    /// Wrap send function for the underlying Sender
    pub fn send(&self, msg: Message) -> async_channel::Send<'_, Message> {
        self.tx.send(msg)
    }
}

impl Stream for Connection {
    type Item = (String, Message);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx
            .poll_next_unpin(cx)
            .map(|opt| opt.map(|msg| (self.metadata.name.clone(), msg)))
    }
}

#[derive(Clone)]
pub struct ComponentChannels {
    // Incoming connections
    pub rx: Vec<Connection>,

    // Named output connections
    pub tx_named: HashMap<String, Connection>,
}
