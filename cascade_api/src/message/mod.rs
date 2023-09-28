use std::collections::HashMap;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

use nanoid::nanoid;

use crate::message::content::Content;

pub mod content;

/// Wraps message to pass signals to the component runtime
pub enum InternalMessage {
    ShutdownSignal,
    Item(Message),
}

/// Message passed to components
#[derive(Debug, Clone)]
pub struct Message {
    pub id: String,
    pub created_nanos: u128,
    // Map of string properties
    pub properties: HashMap<String, String>,
    pub content: HashMap<String, Content>,
}

const DEFAULT_CONTENT_REFERENCE: &str = "default";

impl Message {
    pub fn new(properties: HashMap<String, String>) -> Message {
        Message {
            id: nanoid!(),
            created_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            // No content to begin with
            content: Default::default(),
            properties,
        }
    }

    pub fn new_with_content(properties: HashMap<String, String>, content: Content) -> Message {
        Message {
            id: nanoid!(),
            created_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            // Start with default content populated
            content: HashMap::from([(DEFAULT_CONTENT_REFERENCE.to_string(), content)]),
            properties,
        }
    }
}
