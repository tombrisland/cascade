use std::collections::HashMap;

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::connection::ComponentOutput;
use crate::graph::item::CascadeItem;
use crate::producer::Produce;

const NAME: &str = "GenerateItem";

pub struct GenerateItem {
    // How many items to produce in a single scheduled run
    pub batch_size: i32,
    // Content to include in the items
    pub content: Option<String>,
}

impl Component for GenerateItem {
    fn name(&self) -> &'static str {
        return NAME;
    }
}

#[async_trait]
impl Produce for GenerateItem {
    fn on_initialisation(&self) {}

    async fn try_produce(&self, output: ComponentOutput) -> Result<i32, ComponentError> {
        // Send as many as permitted by batch_size
        for _ in 0..self.batch_size {
            output.send(CascadeItem::new(HashMap::new())).await.unwrap();
        }

        Ok(self.batch_size)
    }
}
