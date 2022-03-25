use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::processor::{Changes, Process, ProcessError};

pub struct LogMessage {
    pub counter: Arc<AtomicUsize>,
    pub control: Control,
}

impl Controllable for LogMessage {
    fn control(&self) -> &Control { &self.control }
}

impl Process for LogMessage {
    fn process(&self, item: FlowItem) -> Result<FlowItem, ProcessError> {
        let count: usize = self.counter.fetch_add(1, Ordering::SeqCst);

        println!("Logging item number {}, {:?}", count, item);

        Result::Ok(item)
    }

    fn will_edit(&self) -> Changes {
        Changes {
            properties: vec![],
            content: false,
        }
    }
}