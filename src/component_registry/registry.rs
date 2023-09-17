use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::processor::Process;
use crate::producer::Produce;

pub type ComponentMap<T> = HashMap<&'static str, fn(Value) -> T>;

#[derive(Clone)]
pub struct ComponentRegistry {
    producers: ComponentMap<Arc<dyn Produce>>,
    processors: ComponentMap<Arc<dyn Process>>,
}

impl ComponentRegistry {
    pub fn new(processors: ComponentMap<Arc<dyn Process>>, producers: ComponentMap<Arc<dyn Produce>>) -> ComponentRegistry {
        ComponentRegistry {
            producers,
            processors,
        }
    }

    pub fn get_producer(&self, type_name: &String, config: Value) -> Option<Arc<dyn Produce>> {
        self.producers.get(type_name.as_str())
            .map(|constructor| constructor.call((config, )))
    }

    pub fn get_processor(&self, type_name: &String, config: Value) -> Option<Arc<dyn Process>> {
        self.processors.get(type_name.as_str())
            .map(|constructor| constructor.call((config, )))
    }

    pub fn is_type_known(&self, type_name: &String) -> bool {
        self.processors.contains_key(type_name.as_str()) || self.producers.contains_key(type_name.as_str())
    }
}