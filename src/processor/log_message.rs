use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use log::info;

use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::processor::{Changes, Process, ProcessError};

const NAME: &str = "LogMessage";

pub struct LogMessage {
    pub counter: AtomicUsize,
    pub control: Control,
}

impl Controllable for LogMessage {
    fn control(&self) -> &Control { &self.control }
}

impl Process for LogMessage {
    fn name(&self) -> &str {
        NAME
    }

    fn process(&self, item: FlowItem) -> Result<FlowItem, ProcessError> {
        let count: usize = self.counter.fetch_add(1, Ordering::SeqCst);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_nanos();

        let elapsed_millis: f64 = (now - item.created_time) as f64 / 1000_000.0;

        info!("Item number {} took {:.2}ms, contents {:?}", count, elapsed_millis, item);

        Result::Ok(item)
    }

    fn will_edit(&self) -> Changes {
        Changes {
            properties: vec![],
            content: false,
        }
    }
}