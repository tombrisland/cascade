// use std::collections::HashMap;
// use std::sync::Arc;
// use std::sync::atomic::AtomicI32;
// use std::sync::mpsc::Sender;
//
// use async_trait::async_trait;
// use futures::future::SelectAll;
// use futures::StreamExt;
// use serde_json::Value;
// use tokio::sync::mpsc::Receiver;
// use tokio_stream::wrappers::ReceiverStream;
//
// use crate::component::NamedComponent;
// use crate::graph::item::CascadeItem;
//
// struct Connection<Payload: Clone> {
//     name: String,
//
//     // Count of current items in the connection
//     item_count: AtomicI32,
//     // Max items queued
//     max_items: u32,
//
//     tx: Sender<Payload>,
//     rx: Receiver<Payload>,
// }
//
// pub struct ComponentExecution<Payload: Clone> {
//     // Stream from all incoming connections
//     incoming_stream: SelectAll<ReceiverStream<Payload>>,
//
//     outgoing_connections: HashMap<String, Connection<Payload>>,
// }
//
// enum ComponentError {
//     InputClosed,
//     OutputClosed,
//     MissingConnection(String),
// }
//
// const DEFAULT_CONNECTION: &str = "default";
//
// impl ComponentExecution<CascadeItem> {
//     // Get a single item from the session
//     pub async fn recv(&mut self) -> Result<CascadeItem, ComponentError> {
//         self.incoming_stream.next().await.unwrap_or_else(|| ComponentError::InputClosed)
//     }
//
//     pub async fn send(&self, name: &str, item: CascadeItem) -> Result<(), ComponentError> {
//         let connection: &Connection<CascadeItem> = self.outgoing_connections.get(name)
//             .ok_or(ComponentError::MissingConnection(name.to_string()))?;
//
//         connection.tx.send(item).or(Err(ComponentError::OutputClosed))
//     }
//
//     // Send to the default connection
//     pub async fn send_default(&self, item: CascadeItem) -> Result<(), ComponentError> {
//         self.send(DEFAULT_CONNECTION, item).await
//     }
// }
//
// #[async_trait]
// pub trait Process<Payload: Clone + Unpin>: NamedComponent + Send + Sync {
//     fn create_from_json(config: Value) -> Arc<dyn Process<Payload>>
//         where
//             Self: Sized;
//
//     async fn process(&self, execution: ComponentExecution<Payload>) -> Result<(), ComponentError>;
// }