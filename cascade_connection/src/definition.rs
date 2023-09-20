use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDefinition {
    pub name: String,
    pub from: usize,
    pub to: usize,

    pub max_items: u32,
}

pub const DEFAULT_CONNECTION: &str = "default";
pub const DEFAULT_MAX_ITEMS: u32 = 1000;

impl ConnectionDefinition {
    pub fn new(from: usize, to: usize) -> Self {
        ConnectionDefinition {
            name: DEFAULT_CONNECTION.to_string(),
            from,
            to,
            max_items: DEFAULT_MAX_ITEMS,
        }
    }
}
