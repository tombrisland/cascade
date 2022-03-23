use std::fmt::{Debug, Formatter};

use crate::Item;

pub struct ProducerConfig {
    pub thread_count: u8,
    pub count_per_second: u32,
}

pub trait Process : Send + Sync {
    fn process(&self, item: Item) -> Result<Item, ProcessError>;
}

pub trait Produce : Send + Sync {
    fn producer_config(&self) -> ProducerConfig;

    fn produce(&self) -> Result<Item, ProduceError>;
}

pub struct ProduceError;

impl Debug for ProduceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProducerError").finish()
    }
}

pub struct ProcessError;

impl Debug for ProcessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessorError").finish()
    }
}