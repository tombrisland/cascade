use std::fmt::{Display, Formatter};
use std::sync::Arc;

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::component::definition::{ComponentDefinition, ComponentType};
use crate::component::{NamedComponent, Process};

pub struct Component {
    pub metadata: ComponentMetadata,
    pub schedule: Schedule,

    // Underlying producer to call
    pub implementation: Arc<dyn Process>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Schedule {
    Unbounded {
        #[serde(default = "concurrency_default")]
        concurrency: u8,
    },
    // Interval {
    //     period_millis: u64,
    //     // Max amount of tasks which can be started to maintain the period
    //     #[serde(default = "concurrency_default")]
    //     max_concurrency: u8,
    // },
    Interval {
        period_millis: u64,
    },
}

fn concurrency_default() -> u8 {
    1
}

#[derive(Clone)]
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

    // Derive metadata from an implementation
    pub fn _from_named<T: NamedComponent>(
        component_type: ComponentType,
        display_name: String,
    ) -> ComponentMetadata {
        let type_name: String = T::type_name().to_string();

        ComponentMetadata {
            id: format!("{}-{}", type_name, nanoid!()),
            type_name,
            display_name,
            component_type,
        }
    }
}
