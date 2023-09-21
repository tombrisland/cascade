use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::ComponentError;
use crate::execution_env::ExecutionEnvironment;

pub mod component;
pub mod definition;
pub mod error;
pub mod execution_env;

/// Implemented by all components to statically defined type name
pub trait NamedComponent {
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Implemented by a either a Producer or Processor component
#[async_trait]
pub trait Process: NamedComponent + Send + Sync {
    fn create_from_json(config: Value) -> Arc<dyn Process>
    where
        Self: Sized;

    async fn process(&self, execution: &mut ExecutionEnvironment) -> Result<(), ComponentError>;
}
