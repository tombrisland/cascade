use std::collections::HashMap;
use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::producer::{Produce, ProduceError, ProducerConfig};

const NAME: &str = "GenerateItem";

pub struct GenerateItem {
    pub content: String,
    pub control: Control,
}

impl Controllable for GenerateItem {
    fn control(&self) -> &Control { &self.control }
}

impl Produce for GenerateItem {
    fn name(&self) -> &str {
        NAME
    }

    fn config(&self) -> ProducerConfig {
        ProducerConfig {
            count_per_second: 100_000,
            retry_count: 0
        }
    }

    fn try_produce(&self) -> Result<FlowItem, ProduceError> {
        Result::Ok(FlowItem::new( HashMap::new() ))
    }
}