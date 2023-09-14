use std::sync::Arc;

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;

pub mod log_message;
pub mod update_properties;

#[async_trait]
// Trait to implement to create a producer
pub trait Process: Component + Send + Sync {
    /// Called when the containing flow is first started.
    /// Any initialisation can be performed here at a low cost.
    fn on_initialisation(&self);

    /// Process a single item and return the result with Ok
    /// In the case of failure a ComponentError should be returned.
    async fn try_process(&self, item: FlowItem) -> Result<FlowItem, ComponentError>;
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