use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::component::{ComponentError, ComponentMetadata, NamedComponent};
use crate::connection::Connections;

pub mod generate_item;

const NSEC_PER_SEC: u32 = 1_000_000_000;

#[async_trait]
// Trait to implement to create a producer
pub trait Produce: NamedComponent + Send + Sync {
    /// Create and return a new instance of this processor
    fn create(config: Value) -> Arc<dyn Produce>
        where
            Self: Sized;

    /// Produce any number of results and forward to the provided channel.
    /// The number of results produced can be returned with Result::Ok.
    /// In the case of failure a ProduceError should be returned.
    async fn produce(&self, outgoing: Connections) -> Result<i32, ComponentError>;
}

// Wrapper for the producer implementation
pub struct Producer {
    pub metadata: ComponentMetadata,
    // Underlying producer to call
    pub implementation: Arc<dyn Produce>,

    pub config: ProducerConfig,

    pub active: AtomicBool,
    pub active_instances: AtomicU32,
}

impl Producer {
    pub fn new(metadata: ComponentMetadata, implementation: Arc<dyn Produce>) -> Producer {
        Producer {
            implementation,
            metadata,
            active: AtomicBool::new(true),
            active_instances: AtomicU32::new(0),
            config: ProducerConfig {
                schedule_per_second: 2,
                concurrency: 1,
            },
        }
    }

    // Indicate that the producer should stop ASAP
    pub fn _request_stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    // Whether the producer should stop
    pub fn should_stop(&self) -> bool {
        !self.active.load(Ordering::SeqCst)
    }

    pub fn increment_concurrency(&self) {
        self.active_instances.fetch_add(1, Ordering::SeqCst);
    }

    pub fn decrement_concurrency(&self) {
        self.active_instances.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn concurrency_maxed(&self) -> bool {
        self.active_instances.load(Ordering::SeqCst) >= self.config.concurrency
    }
}

// Configuration for a producer instance
pub struct ProducerConfig {
    /// How often to schedule the producer for running
    /// Best effort as system resources may mean speed is not attained
    pub schedule_per_second: u32,

    // Maximum amount of instances that can be in progress at the same time
    pub concurrency: u32,
}

impl ProducerConfig {
    pub fn schedule_duration(&self) -> Duration {
        // How often should be producer be scheduled
        let interval: u64 = (NSEC_PER_SEC / self.schedule_per_second) as u64;

        Duration::from_nanos(interval)
    }
}