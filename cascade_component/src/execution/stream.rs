use async_channel::Receiver;
use futures::stream::{select_all, SelectAll};
use futures::StreamExt;

type FusedStream<T> = SelectAll<Receiver<T>>;

/// Wraps async-channel receivers to create a fused stream
/// These channels can then be read from the same source
pub struct ReceiverStream<Message> {
    select_all: FusedStream<Message>,
}

impl<Message> ReceiverStream<Message> {
    pub fn new(receivers: Vec<Receiver<Message>>) -> ReceiverStream<Message> {
        ReceiverStream {
            select_all: select_all(receivers),
        }
    }

    pub(crate) async fn recv(&mut self) -> Option<Message> {
        self.select_all.next().await
    }
}
