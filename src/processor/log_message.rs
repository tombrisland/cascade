use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use log::info;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;
use crate::processor::{Changes, Process};

const NAME: &str = "LogMessage";

pub struct LogMessage {
    // Count of the items passed through this processor
    pub item_count: AtomicUsize,

    // Only log every x results
    pub log_every_x: usize,
}

impl Component for LogMessage {
    fn name(&self) -> &'static str {
        NAME
    }
}

#[async_trait]
impl Process for LogMessage {
    fn on_initialisation(&self) {}

    async fn try_process(&self, item: FlowItem) -> Result<FlowItem, ComponentError> {
        // Increment item count and fetch the value
        let count: usize = self.item_count.fetch_add(1, Ordering::SeqCst);

        // Only log every x results
        if count % self.log_every_x == 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH).unwrap().as_nanos();

            let elapsed_millis: f64 = (now - item.created_time) as f64 / 1_000_000.0;

            info!("Item number {} took {:.2}ms, contents {:?}", count, elapsed_millis, item);
        }

        Result::Ok(item)
    }

    fn will_edit(&self) -> Changes {
        Changes {
            properties: vec![],
            content: false,
        }
    }
}