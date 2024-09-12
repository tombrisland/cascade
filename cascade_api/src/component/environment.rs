use crate::component::error::ComponentError;
use crate::message::Message;
use async_trait::async_trait;

#[async_trait]
pub trait ComponentEnvironment: Send + Sync {
    /// Get a single item from any of the input queues
    async fn recv<'a>(&'a mut self) -> Result<&'a Message, ComponentError>;

    /// Send an item to a name output connection
    async fn send(&mut self, name: &str, item: Message) -> Result<(), ComponentError>;

    /// Send an item to the default output connection
    async fn send_default(&mut self, item: Message) -> Result<(), ComponentError>;
}
