use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use nanoid::nanoid;

use crate::component::Controllable;
use crate::flow::item::FlowItem;
use crate::processor::Process;

pub mod generate_item;
pub mod get_file;

const NSEC_PER_SEC: u32 = 1_000_000_000;

pub struct Producer {
    // Unique to this producer instance
    pub id: String,

    stop_requested: AtomicBool,

    // Underlying producer to call
    producer: Arc<dyn Produce>,
}

impl Producer {
    pub fn new(producer: Arc<dyn Produce>) -> Producer {
        let id = format!("{}-{}", producer.name(), nanoid!());

        Producer {
            id,
            stop_requested: AtomicBool::new(false),
            producer,
        }
    }

    pub fn config(&self) -> ProducerConfig {
        return self.producer.config();
    }

    // Return a new pointer to the underlying producer impl
    pub fn producer(&self) -> Arc<dyn Produce> {
        self.producer.clone()
    }

    pub fn stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    pub fn request_stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub struct ProducerConfig {
    // The run count per second for each instance
    pub count_per_second: u32,

    // Retries before termination
    pub retry_count: u8,
}

pub trait Produce: Send + Sync + Controllable {
    fn name(&self) -> &str;

    fn config(&self) -> ProducerConfig;

    fn try_produce(&self) -> Result<FlowItem, ProduceError>;
}

impl ProducerConfig {
    pub fn schedule_duration(&self) -> Duration {
        // How often should be producer be invoked
        let interval: u64 = (NSEC_PER_SEC / self.count_per_second) as u64;

        Duration::from_nanos(interval)
    }
}

#[derive(Debug)]
pub struct ProduceError;

impl Display for ProduceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "")
    }
}