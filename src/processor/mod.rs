use std::collections::HashMap;
use std::future::Future;
use std::sync::mpsc::{SendError, Sender};
use std::sync::Arc;

use async_trait::async_trait;
use tokio::join;
use tokio::task::JoinHandle;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;

pub mod log_message;
pub mod update_properties;

const BATCH_SIZE: i8 = 1;
const MERGES_FLOW: bool = false;

#[async_trait]
// Trait to implement to create a producer
pub trait Process: Component + Send + Sync {
    /// Called when the containing flow is first started.
    /// Any initialisation can be performed here at a low cost.
    fn on_initialisation(&self);

    /// Process a single item and return the result with Result::Ok
    /// In the case of failure a ComponentError should be returned.
    async fn try_process(&self, mut item: FlowItem) -> Result<FlowItem, ComponentError>;

    /// Declare what this processor might edit.
    /// It's important to get this right as it may cause execution flow to change.
    fn will_edit(&self) -> Changes;
}

// Wrapper for the processor implementation
pub struct Processor {
    // Underlying processor to call
    pub implementation: Arc<dyn Process>,
}

impl Processor {
    pub fn new<T: 'static + Process>(implementation: T) -> Processor {
        Processor {
            implementation: Arc::new(implementation),
        }
    }
}

// Declaration of potential changes to a FlowItem
pub struct Changes {
    pub properties: Vec<String>,
    pub content: bool,
}

pub struct ProcessorConfig {
    // The max size of items to pass in one go
    batch_size: i8,

    // Whether the processor will merge multiple upstream flows
    merges_flow: bool,

    // Other processor specific properties
    properties: HashMap<String, String>,
}

impl ProcessorConfig {
    fn new(properties: HashMap<String, String>) -> ProcessorConfig {
        ProcessorConfig {
            batch_size: BATCH_SIZE,
            merges_flow: MERGES_FLOW,
            properties,
        }
    }
}

pub struct ProcessorResult {
    handle: Option<Arc<JoinHandle<FlowItem>>>,
    item: Option<FlowItem>,
}

impl ProcessorResult {
    pub fn from_handle(handle: JoinHandle<FlowItem>) -> ProcessorResult {
        ProcessorResult {
            handle: Some(Arc::new(handle)),
            item: None,
        }
    }

    pub fn from_item(item: FlowItem) -> ProcessorResult {
        ProcessorResult {
            handle: None,
            item: Some(item),
        }
    }

    pub async fn item(mut self) -> FlowItem {
        match self.item {
            // Return a clone of the cached item
            Some(item) => item,
            // Wait for the handle to join and return the item
            None => FlowItem::new(HashMap::new()),
        }
    }
}
