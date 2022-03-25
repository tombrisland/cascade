use std::fmt::{Debug, Formatter};

use crate::component::Controllable;
use crate::flow::item::FlowItem;

pub mod update_properties;
pub mod log_message;

pub struct Changes {
    pub properties: Vec<String>,
    pub content: bool,
}

pub trait Process: Send + Sync + Controllable {
    fn process(&self, item: FlowItem) -> Result<FlowItem, ProcessError>;

    fn will_edit(&self) -> Changes;
}

#[derive(Debug)]
pub struct ProcessError;