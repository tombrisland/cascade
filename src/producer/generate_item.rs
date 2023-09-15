use std::collections::HashMap;

use async_trait::async_trait;

use crate::component::{ComponentError, NamedComponent};
use crate::connection::ComponentOutput;
use crate::graph::item::CascadeItem;
use crate::producer::Produce;

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
    async fn produce(&self, output: ComponentOutput) -> Result<i32, ComponentError> {
        // Send as many as permitted by batch_size
        for _ in 0..self.batch_size {
            output.send(CascadeItem::new(HashMap::new())).await.unwrap();
        }

        Ok(self.batch_size)
    }
}
