use std::collections::HashMap;
use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::producer::{Produce, ProduceError, ProducerConfig};

pub struct GenerateItem {
    pub content: String,
    pub control: Control,
}

impl Controllable for GenerateItem {
    fn control(&self) -> &Control { &self.control }
}

impl Produce for GenerateItem {
    fn producer_config(&self) -> ProducerConfig {
        ProducerConfig {
            thread_count: 1,
            count_per_second: 1
        }
    }

    fn try_produce(&self) -> Result<FlowItem, ProduceError> {
        Result::Ok(FlowItem::new( HashMap::new() ))
    }
}