use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::component::component::{Component, ComponentMetadata, Schedule};
use crate::component::definition::{ComponentDefinition, ComponentType};
use crate::component::Process;

pub type ComponentMap = HashMap<&'static str, fn(Value) -> Arc<dyn Process>>;

#[derive(Clone)]
pub struct ComponentRegistry {
    components: ComponentMap,
}

impl ComponentRegistry {
    pub fn new(components: ComponentMap) -> ComponentRegistry {
        ComponentRegistry { components }
    }

    pub fn get_component(&self, def: &ComponentDefinition) -> Option<Component> {
        let metadata: ComponentMetadata = ComponentMetadata::from_def(def);

        // Retrieve implementation from registry if present
        let implementation: Arc<dyn Process> = self
            .components
            .get(metadata.type_name.as_str())
            .map(|constructor| constructor.call((def.config.clone(), )))?;

        let schedule: Schedule = match metadata.component_type {
            ComponentType::Producer => Schedule::Interval(1000),
            ComponentType::Processor => Schedule::Unbounded,
        };

        Some(Component {
            metadata,
            schedule,
            implementation,
        })
    }
    pub fn is_known_component(&self, type_name: &String) -> bool {
        self.components.contains_key(type_name.as_str())
    }
}
