use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;
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

    async fn try_produce(&self, tx: Sender<FlowItem>) -> Result<Option<i32>, ComponentError> {
        // Send as many as permitted by batch_size
        for _ in 0..self.batch_size {
            tx.send(FlowItem::new(HashMap::new())).await;
        }

        Result::Ok(Option::Some(self.batch_size))
    }
}
