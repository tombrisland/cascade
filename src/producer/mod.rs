use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use async_trait::async_trait;

use crate::component::{Component, ComponentError, NamedComponent};
use crate::connection::ComponentOutput;

pub mod generate_item;

const NSEC_PER_SEC: u32 = 1_000_000_000;

#[async_trait]
// Trait to implement to create a producer
pub trait Produce: NamedComponent + Send + Sync {
    /// Produce any number of results and forward to the provided channel.
    /// The number of results produced can be returned with Result::Ok.
    /// In the case of failure a ProduceError should be returned.
    async fn produce(&self, outgoing: ComponentOutput) -> Result<i32, ComponentError>;
}

// Wrapper for the producer implementation
pub struct Producer {
    // Underlying producer to call
    pub implementation: Box<dyn Produce>,

    pub component: Component,
    pub config: ProducerConfig,

    pub active: AtomicBool,
    pub active_instances: AtomicU32,
}

impl Producer {
    pub fn new<T: 'static + Produce>(implementation: T, config: ProducerConfig) -> Producer {
        Producer {
            implementation: Box::new(implementation),
            component: Component::new::<T>("display_name".to_string()),
            active: AtomicBool::new(true),
            active_instances: AtomicU32::new(0),
            config,
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