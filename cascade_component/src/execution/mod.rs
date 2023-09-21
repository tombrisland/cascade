pub mod environment;
mod stream;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::error;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{interval, Interval};
use tokio::time::MissedTickBehavior::Delay;

use cascade_connection::{ComponentChannels, Message};
use crate::component::{Component, ComponentMetadata, Schedule};
use crate::error::ComponentError;
use crate::execution::environment::ExecutionEnvironment;
use crate::Process;

pub struct ComponentExecution {
    // Active task for this execution
    tasks: JoinSet<()>,

    pub component: Arc<Component>,

    shutdown: Arc<AtomicBool>,
    channels: ComponentChannels,
}

impl ComponentExecution {
    pub fn new(component: Component, channels: ComponentChannels) -> ComponentExecution {
        ComponentExecution {
            tasks: JoinSet::new(),
            component: Arc::new(component),
            shutdown: Default::default(),
            channels,
        }
    }

    pub fn start(&mut self) {
        let metadata: ComponentMetadata = self.component.metadata.clone();

        match self.component.schedule {
            // Allow the component to manage it's own scheduling
            Schedule::Unbounded { concurrency } => {
                for _ in 0..concurrency {
                    let environment: ExecutionEnvironment =
                        ExecutionEnvironment::new(metadata.clone(), self.channels.clone());

                    self.schedule_component(environment, None);
                }
            }
            // Schedule the component at set intervals
            Schedule::Interval { period_millis } => {
                let mut interval: Interval = interval(Duration::from_millis(period_millis));
                // Don't try and catch up with missed ticks
                interval.set_missed_tick_behavior(Delay);

                let environment: ExecutionEnvironment =
                    ExecutionEnvironment::new(metadata.clone(), self.channels.clone());

                self.schedule_component(environment, Some(interval));
            }
        };
    }

    pub async fn stop(&mut self) -> Result<(), JoinError> {
        self.shutdown.store(true, Ordering::Relaxed);

        // Wait for all tasks to stop
        loop {
            self.channels
                .tx_signal
                .send(Message::ShutdownSignal)
                .await
                .unwrap();

            match self.tasks.join_next().await {
                // Finished joining tasks
                None => break,
                Some(res) => res?,
            }
        }

        Ok(())
    }

    fn schedule_component(
        &mut self,
        mut environment: ExecutionEnvironment,
        mut interval: Option<Interval>,
    ) {
        let implementation: Arc<dyn Process> = self.component.implementation.clone();
        let shutdown: Arc<AtomicBool> = self.shutdown.clone();

        self.tasks.spawn(async move {
            loop {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }

                // Enable shutdown for components which don't read input
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                if let Err(err) = implementation.process(&mut environment).await {
                    match err {
                        ComponentError::ComponentShutdown => {
                            // Break loop and join task
                            break;
                        }
                        err => error!("Encountered error with {:?}", err),
                    }
                }
            }
        });
    }
}
