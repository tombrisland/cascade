use std::fmt::{Debug, Formatter};
use std::sync::atomic::AtomicBool;
use crate::component::{Control, Controllable};

use crate::flow::item::FlowItem;
use crate::processor::Process;

pub mod generate_item;
pub mod get_file;

pub struct ProducerConfig {
    pub thread_count: u8,
    pub count_per_second: u32,
}

pub trait Produce : Send + Sync + Controllable {
    fn producer_config(&self) -> ProducerConfig;

    fn try_produce(&self) -> Result<FlowItem, ProduceError>;
}

#[derive(Debug)]
pub struct ProduceError;