use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_channel::{Receiver, Sender};

use cascade_connection::definition::DEFAULT_CONNECTION;
use cascade_payload::CascadeItem;

use crate::component::ComponentMetadata;
use crate::error::ComponentError;

pub struct ExecutionEnvironment {
    pub metadata: ComponentMetadata,

    // Connections which can be ignored if they don't exist
    pub drop_connections: Vec<String>,

    rx: Vec<Arc<Receiver<CascadeItem>>>,
    // Enable round robin of the input queues
    rx_idx: AtomicUsize,

    tx_named: HashMap<String, Arc<Sender<CascadeItem>>>,
}

impl ExecutionEnvironment {
    pub async fn new(
        metadata: ComponentMetadata,
        rx: Vec<Arc<Receiver<CascadeItem>>>,
        tx_named: HashMap<String, Arc<Sender<CascadeItem>>>,
    ) -> ExecutionEnvironment {
        // TODO could error here if rel not available
        ExecutionEnvironment {
            metadata,
            drop_connections: vec![DEFAULT_CONNECTION.to_string()],
            rx,
            rx_idx: Default::default(),
            tx_named,
        }
    }

    // Get a single item from the session
    pub async fn recv(&self) -> Result<CascadeItem, ComponentError> {
        let rx_idx: usize = self.rx_idx.load(Ordering::Relaxed);

        self.rx_idx.store(
            if rx_idx == self.rx.len() - 1 {
                0
            } else {
                rx_idx + 1
            },
            Ordering::Relaxed,
        );

        self.rx
            .get(rx_idx)
            .unwrap()
            .recv()
            .await
            .map_err(|_| ComponentError::InputClosed)
    }

    pub async fn send(&self, name: &str, item: CascadeItem) -> Result<(), ComponentError> {
        match self.tx_named.get(name) {
            Some(connection) => connection
                .send(item)
                .await
                .or(Err(ComponentError::OutputClosed)),
            None => {
                // Only error if there are connections that aren't configured to drop
                if self.tx_named.is_empty() || self.drop_connections.contains(&name.to_string()) {
                    Ok(())
                } else {
                    Err(ComponentError::MissingOutputConnection(name.to_string()))
                }
            }
        }
    }

    // Send to the default connection
    pub async fn send_default(&self, item: CascadeItem) -> Result<(), ComponentError> {
        self.send(DEFAULT_CONNECTION, item).await
    }
}
