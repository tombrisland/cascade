use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use log::warn;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FlowItem {
    pub id: uuid::Uuid,
    // Time the item was created in epoch nanos
    pub created_time: u128,
    // Map of string properties
    pub properties: HashMap<String, String>,
    pub content: Option<Vec<u8>>,
    // Optional id of the parent flow item
    pub parent_id: Option<uuid::Uuid>,
}

impl FlowItem {
    pub fn new(properties: HashMap<String, String>) -> FlowItem {
        FlowItem {
            // TODO clone method should set parent_id and replace id
            id: Uuid::new_v4(),
            created_time: SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap().as_nanos(),
            // No content to begin with
            content: None,
            properties,
            parent_id: None
        }
    }
}

pub struct FlowSession {
    items: Vec<FlowItem>
}

impl FlowSession {
    fn item(&self) -> &FlowItem {
        if self.items.len() != 1 {
            panic!("Item called on session with more than 1 item");
        }

        return self.items.first().unwrap();
    }

    fn items_iterator() {

    }
}