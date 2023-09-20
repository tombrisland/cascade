use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use cascade_component::{NamedComponent, Process};
use cascade_component::error::ComponentError;
use cascade_component::execution_env::ExecutionEnvironment;
use cascade_payload::CascadeItem;

#[derive(Serialize, Deserialize)]
pub struct LogMessageConfig {
    // Only log every x results
    pub log_every_x: usize,
}

pub struct LogMessage {
    config: LogMessageConfig,
    // Count of the items passed through this processor
    pub item_count: AtomicUsize,
}

impl NamedComponent for LogMessage {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "LogMessage"
    }
}

#[async_trait]
impl Process for LogMessage {
    fn create_from_json(config: Value) -> Arc<dyn Process> {
        let config: LogMessageConfig = serde_json::from_value(config).unwrap();

        Arc::new(LogMessage {
            config,
            item_count: Default::default(),
        })
    }

    async fn process(&self, execution: &mut ExecutionEnvironment) -> Result<(), ComponentError> {
        let item: CascadeItem = execution.recv().await?;

        // Increment item count and fetch the value
        let count: usize = self.item_count.fetch_add(1, Ordering::SeqCst);

        // Only log every x results
        if count % self.config.log_every_x == 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            let elapsed_millis: f64 = (now - item.created_nanos) as f64 / 1_000_000.0;

            info!(
                "Item number {} took {:.2}ms, contents {:?}",
                count, elapsed_millis, item
            );
        }

        execution.send_default(item).await
    }
}
