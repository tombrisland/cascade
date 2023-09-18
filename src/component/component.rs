use std::sync::Arc;

use nanoid::nanoid;

use crate::component::definition::{ComponentDefinition, ComponentType};
use crate::component::Process;

pub struct Component {
    pub metadata: ComponentMetadata,
    pub schedule: Schedule,

    // Underlying producer to call
    pub implementation: Arc<dyn Process>,
}

pub enum Schedule {
    Unbounded,
    Interval(u64),
}

#[derive(Clone)]
pub struct ComponentMetadata {
    pub id: String,

    pub type_name: String,
    pub display_name: String,
    // Whether this is a processor or producer
    pub component_type: ComponentType,
}

impl ComponentMetadata {
    pub fn from_def(def: &ComponentDefinition) -> ComponentMetadata {
        let type_name: String = def.type_name.clone();

        ComponentMetadata {
            id: format!("{}-{}", type_name, nanoid!()),
            type_name,
            display_name: def.display_name.clone(),
            component_type: def.component_type.clone(),
        }
    }

    // pub fn from_named<T: NamedComponent>(display_name: String) -> ComponentMetadata {
    //     let type_name: String = T::type_name().to_string();
    //
    //     ComponentMetadata {
    //         id: format!("{}-{}", type_name, nanoid!()),
    //         type_name,
    //         display_name,
    //     }
    // }
}
