use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;

use async_trait::async_trait;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;

pub mod generate_item;
pub mod get_file;

const NSEC_PER_SEC: u32 = 1_000_000_000;

#[async_trait]
// Trait to implement to create a producer
pub trait Produce: Component + Send + Sync {
    /// Called when the containing flow is first started.
    /// Any initialisation can be performed here at a low cost.
    fn on_initialisation(&self);

    /// Produce any number of results and forward to the provided channel.
    /// The number of results produced can be returned with Result::Ok.
    /// In the case of failure a ProduceError should be returned.
    async fn try_produce(&self, tx: Sender<FlowItem>) -> Result<Option<i32>, ComponentError>;
}

// Wrapper for the producer implementation
pub struct Producer {
    // Underlying producer to call
    pub implementation: Box<dyn Produce>,

    pub active: AtomicBool,

    pub config: ProducerConfig,
}

impl Producer {
    pub fn new<T: 'static + Produce>(implementation: T, config: ProducerConfig) -> Producer {
        Producer {
            implementation: Box::new(implementation),
            active: AtomicBool::new(true),
            config,
        }
    }

    // Indicate that the producer should stop ASAP
    pub fn request_stop(&self) {
        self.active.store(false, Ordering::SeqCst);
    }

    // Whether the producer should stop
    pub fn should_stop(&self) -> bool {
        !self.active.load(Ordering::SeqCst)
    }
}

// Configuration for a producer instance
pub struct ProducerConfig {
    /// How often to schedule the producer for running
    /// Best effort as system resources may mean speed is not attained
    pub schedule_per_second: u32,
}

impl ProducerConfig {
    pub fn schedule_duration(&self) -> Duration {
        // How often should be producer be scheduled
        let interval: u64 = (NSEC_PER_SEC / self.schedule_per_second) as u64;

        Duration::from_nanos(interval)
    }
}