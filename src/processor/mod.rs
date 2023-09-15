use std::sync::Arc;

use async_trait::async_trait;

use crate::component::{Component, ComponentError, NamedComponent};
use crate::graph::item::CascadeItem;

pub mod log_message;
pub mod update_properties;

#[async_trait]
// Trait to implement to create a producer
pub trait Process: NamedComponent + Send + Sync {
    /// Process a single item and return the result with Ok
    /// In the case of failure a ComponentError should be returned.
    async fn process(&self, item: CascadeItem) -> Result<CascadeItem, ComponentError>;
}

// Wrapper for the processor implementation
pub struct Processor {
    // Underlying processor to call
    pub implementation: Arc<dyn Process>,
    pub component: Component,
}

impl Processor {
    pub fn new<T: 'static + Process>(implementation: T) -> Processor {
        Processor {
            implementation: Arc::new(implementation),
            component: Component::new::<T>("display_name".to_string()),
        }
    }
}