use std::fmt::{Display, Formatter};

use crate::component::definition::{ComponentDefinition, ComponentType};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Schedule {
    Unbounded {
        #[serde(default = "concurrency_default")]
        concurrency: u8,
    },
    Interval {
        period_millis: u64,
    },
}

fn concurrency_default() -> u8 {
    1
}

#[derive(Clone, Serialize)]
pub struct ComponentMetadata {
    pub id: String,

    pub type_name: String,
    pub display_name: String,
    // Whether this is a processor or producer
    pub component_type: ComponentType,
}

// Used as a prefix in other logs for traceability
impl Display for ComponentMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "[{:?}:{}:{}]",
            self.component_type, self.type_name, self.id
        ))
    }
}

impl ComponentMetadata {
    pub fn from_def(def: &ComponentDefinition) -> ComponentMetadata {
        let type_name: String = def.type_name.clone();

        ComponentMetadata {
            id: nanoid!(),
            type_name,
            display_name: def.display_name.clone(),
            component_type: def.component_type.clone(),
        }
    }
}
