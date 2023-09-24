use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDefinition {
    #[serde(default = "id_default", skip_deserializing)]
    pub id: String,

    pub name: String,
    pub source: usize,
    pub target: usize,

    pub max_items: usize,
}

fn id_default() -> String {
    nanoid!()
}

pub const DEFAULT_CONNECTION: &str = "default";
pub const DEFAULT_MAX_ITEMS: usize = 1000;

impl ConnectionDefinition {
    pub fn new(from: usize, to: usize) -> Self {
        ConnectionDefinition {
            id: id_default(),
            name: DEFAULT_CONNECTION.to_string(),
            source: from,
            target: to,
            max_items: DEFAULT_MAX_ITEMS,
        }
    }
}
