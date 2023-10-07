use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::error;
use tokio::task::{JoinError, JoinSet};
use tokio::time::{interval, Interval};
use tokio::time::MissedTickBehavior::Delay;

use cascade_api::component::component::{Component, ComponentMetadata, Schedule};
use cascade_api::component::environment::ExecutionEnvironment;
use cascade_api::component::error::ComponentError;
use cascade_api::component::Process;
use cascade_api::connection::ComponentChannels;
use cascade_api::message::InternalMessage;

pub struct ComponentExecution {
    // Active task for this execution
    tasks: JoinSet<()>,

    pub component: Arc<Component>,

    stopped: Arc<AtomicBool>,
    channels: ComponentChannels,
}

impl ComponentExecution {
    pub fn new(component: Component, channels: ComponentChannels) -> ComponentExecution {
        ComponentExecution {
            tasks: JoinSet::new(),
            component: Arc::new(component),
            stopped: Default::default(),
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
        self.stopped.store(true, Ordering::Relaxed);

        for _ in 0..self.tasks.len() {
            // Send a cancellation message to the thread
            self.channels
                .tx_signal
                .send(InternalMessage::ShutdownSignal)
                .await
                .unwrap();
        }



        Ok(())
    }

    pub async fn kill(&mut self) {
        self.tasks.shutdown().await
    }

    fn schedule_component(
        &mut self,
        mut environment: ExecutionEnvironment,
        mut interval: Option<Interval>,
    ) {
        let implementation: Arc<dyn Process> = self.component.implementation.clone();
        let stopped: Arc<AtomicBool> = self.stopped.clone();

        self.tasks.spawn(async move {
            loop {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }

                if stopped.load(Ordering::Relaxed) {
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
