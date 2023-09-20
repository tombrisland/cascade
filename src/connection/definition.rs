use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDefinition {
    pub name: String,
    pub from: usize,
    pub to: usize,

    pub max_items: u32,
}
