use std::fmt::{Display, Formatter};
use std::sync::mpsc::SendError;
use nanoid::nanoid;
use petgraph::graph::NodeIndex;
use crate::flow::item::FlowItem;

// Trait implemented by producers and processors alike
pub trait Component {
    // Return a simple name for the component
    fn name(&self) -> &'static str;

    // Create a unique id derived from the name
    fn id(&self) -> String {
        format!("{}-{}", self.name(), nanoid!())
    }
}

// Error returned if a component fails
pub struct ComponentError {
    name: String,

    // Messages describing the error
    msg: String,
    detail: Option<String>
}

impl Display for ComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            None => { write!(f, "Component {} {}", self.name, self.msg) }
            Some(detail) => { write!(f, "Component {} {} with {}", self.name, self.msg, detail.to_string()) }
        }
    }
}

impl ComponentError {
    // Create a new error from within a Component
    pub fn new<T : Component>(component: &T, msg: String) -> ComponentError {
        ComponentError {
            name: component.name().to_string(),
            msg,
            detail: None
        }
    }

    // Derive an error from failure to send to an output channel
    pub fn from_send_error<T : Component>(component: &T, err: SendError<FlowItem>) -> ComponentError {
        ComponentError {
            name: component.name().to_string(),
            msg: err.to_string(),
            detail: Option::Some("could not send to channel".to_string())
        }
    }
}