use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::component::{ComponentError, ComponentMetadata, NamedComponent};
use crate::graph::item::CascadeItem;

pub mod log_message;
pub mod update_properties;

#[async_trait]
// Trait to implement to create a producer
pub trait Process: NamedComponent + Send + Sync {
    /// Create and return a new instance of this processor
    fn create(config: Value) -> Arc<dyn Process>
        where
            Self: Sized;

    /// Process a single item and return the result with Ok
    /// In the case of failure a ComponentError should be returned.
    async fn process(&self, item: CascadeItem) -> Result<CascadeItem, ComponentError>;
}

// Wrapper for the processor implementation
pub struct Processor {
    pub metadata: ComponentMetadata,
    // Underlying processor to call
    pub implementation: Arc<dyn Process>,
}

impl Processor {
    pub fn new(metadata: ComponentMetadata, implementation: Arc<dyn Process>) -> Processor {
        Processor {
            implementation,
            metadata,
        }
    }
}