use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use nanoid::nanoid;

use crate::component::Controllable;
use crate::flow::item::FlowItem;

pub mod update_properties;
pub mod log_message;

pub struct Processor {
    // Unique to this processor instance
    pub id: String,

    // Underlying processor to call
    processor: Arc<dyn Process>,
}


impl Processor {
    pub fn new(processor: Arc<dyn Process>) -> Processor {
        let id = format!("{}-{}", processor.name(), nanoid!());

        Processor {
            id,
            processor,
        }
    }

    // Return a new pointer to the underlying producer impl
    pub fn processor(&self) -> Arc<dyn Process> {
        self.processor.clone()
    }
}

pub struct Changes {
    pub properties: Vec<String>,
    pub content: bool,
}

pub trait Process: Send + Sync + Controllable {
    fn name(&self) -> &str;

    fn process(&self, item: FlowItem) -> Result<FlowItem, ProcessError>;

    fn will_edit(&self) -> Changes;
}

#[derive(Debug)]
pub struct ProcessError;