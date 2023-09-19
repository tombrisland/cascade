use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component::{NamedComponent, Process};
use crate::component::error::ComponentError;
use crate::component::execution::ExecutionEnvironment;
use crate::graph::item::CascadeItem;

#[derive(Serialize, Deserialize)]
pub struct GenerateItem {
    // How many items to produce in a single scheduled run
    pub batch_size: i32,
    // Content to include in the items
    pub content: Option<String>,
}

impl NamedComponent for GenerateItem {
    fn type_name() -> &'static str where Self: Sized {
        "GenerateItem"
    }
}

#[async_trait]
impl Process for GenerateItem {
    fn create_from_json(config: Value) -> Arc<dyn Process>
    {
        let generate_item: GenerateItem = serde_json::from_value(config).unwrap();

        Arc::new(generate_item)
    }

    async fn process(&self, execution: &mut ExecutionEnvironment) -> Result<(), ComponentError> {
        // Send as many as permitted by batch_size
        for _ in 0..self.batch_size {
            execution.send_default(CascadeItem::new(HashMap::new())).await.unwrap();
        }

        Ok(())
    }
}