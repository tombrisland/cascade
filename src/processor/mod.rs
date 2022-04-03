use std::sync::Arc;
use std::sync::mpsc::{Sender, SendError};

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;

pub mod update_properties;
pub mod log_message;

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

pub struct Changes {
    pub properties: Vec<String>,
    pub content: bool,
}