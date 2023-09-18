use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::component::error::ComponentError;
use crate::component::execution::ComponentEnv;

pub mod component;
pub mod definition;
pub mod error;
pub mod execution;

pub trait NamedComponent {
    fn type_name() -> &'static str
        where
            Self: Sized;
}

#[async_trait]
pub trait Process: NamedComponent + Send + Sync {
    fn create_from_json(config: Value) -> Arc<dyn Process>
        where
            Self: Sized;

    async fn process(&self, execution: &mut ComponentEnv) -> Result<(), ComponentError>;
}
