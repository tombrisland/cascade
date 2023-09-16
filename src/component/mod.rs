use std::fmt::{Display, Formatter};

use nanoid::nanoid;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tokio::sync::mpsc::error::SendError;

use crate::graph::item::CascadeItem;
use crate::processor::Processor;
use crate::producer::Producer;

mod execution;

pub trait NamedComponent {
    fn type_name() -> &'static str where Self: Sized;
}

#[derive(Clone, Deserialize)]
pub enum ComponentType {
    Producer,
    Processor,
}

#[derive(Clone, Deserialize)]
pub struct ComponentDefinition {
    pub metadata: ComponentMetadata,

    pub component_type: ComponentType,

    pub config: Value,
}

#[derive(Clone, Deserialize)]
pub struct ComponentMetadata {
    pub id: String,

    pub type_name: String,
    pub display_name: String,
}

pub enum ComponentInstance {
    Producer(Producer),
    Processor(Processor),
}

impl ComponentMetadata {
    pub fn from_named_impl<T: NamedComponent>(display_name: String) -> ComponentMetadata {
        let type_name: String = T::type_name().to_string();

        ComponentMetadata {
            id: format!("{}-{}", type_name, nanoid!()),
            type_name,
            display_name,
        }
    }
}

#[derive(Debug)]
pub struct ComponentError {
    message: String,
    can_retry: bool,
}

impl Display for ComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error with {} can_retry {}", self.message, self.can_retry))
    }
}

impl ComponentError {
    // Create a new error from within a Component
    pub fn _new(message: String) -> ComponentError {
        ComponentError {
            message,
            can_retry: false,
        }
    }

    // Derive an error from failure to send to an output channel
    pub fn _from_send_error(
        _err: SendError<CascadeItem>,
    ) -> ComponentError {
        ComponentError {
            message: "could not send to channel".to_string(),
            can_retry: true,
        }
    }
}
