use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component::Schedule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    Producer,
    Processor,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    pub display_name: String,

    pub type_name: String,
    pub component_type: ComponentType,
    #[serde(default = "schedule_default")]
    pub schedule: Schedule,

    pub config: Value,
}

fn schedule_default() -> Schedule {
    Schedule::Interval{
        // Default to once every half second
        period_millis: 500,
    }
}