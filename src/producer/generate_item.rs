use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component::{ComponentError, NamedComponent};
use crate::connection::Connections;
use crate::graph::item::CascadeItem;
use crate::producer::Produce;

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
impl Produce for GenerateItem {
    fn create(config: Value) -> Arc<dyn Produce>
    {
        let generate_item: GenerateItem = serde_json::from_value(config).unwrap();

        Arc::new(generate_item)
    }

    async fn produce(&self, output: Connections) -> Result<i32, ComponentError> {
        // Send as many as permitted by batch_size
        for _ in 0..self.batch_size {
            output.send(CascadeItem::new(HashMap::new())).await.unwrap();
        }

        Ok(self.batch_size)
    }
}
