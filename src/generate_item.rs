use crate::component::{Produce, ProduceError, ProducerConfig};
use crate::Item;

pub struct GenerateItem {
    pub content: String,
}

impl Produce for GenerateItem {
    fn producer_config(&self) -> ProducerConfig {
        ProducerConfig {
            thread_count: 1,
            count_per_second: 1
        }
    }

    fn produce(&self) -> Result<Item, ProduceError> {
        Result::Ok(Item::new())
    }
}