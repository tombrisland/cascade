use std::fmt::{Display, Formatter};

use nanoid::nanoid;
use tokio::sync::mpsc::error::SendError;

use crate::graph::item::CascadeItem;

// Trait implemented by producers and processors alike
pub trait NamedComponent {
    fn type_name() -> &'static str where Self: Sized;
}

pub struct Component {
    pub id: String,

    pub type_name: &'static str,
    pub display_name: String,
}

impl Component {
    pub fn new<T: NamedComponent>(display_name: String) -> Component {
        let type_name: &str = T::type_name();

        Component {
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
