use log::error;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::{JoinSet};
use tokio::time::MissedTickBehavior::Delay;
use tokio::time::{interval, Interval};

use cascade_api::component::component::{Component, ComponentMetadata, Schedule};
use cascade_api::component::environment::{ExecutionEnvironment, StopComponent};
use cascade_api::component::error::ComponentError;
use cascade_api::component::Process;
use cascade_api::connection::ComponentChannels;

pub struct ComponentExecution {
    // Active task for this execution
    tasks: JoinSet<()>,

    pub component: Arc<Component>,

    pub stop_component: Option<Arc<StopComponent>>,
    channels: ComponentChannels,
}

impl ComponentExecution {
    pub fn new(component: Component, channels: ComponentChannels) -> ComponentExecution {
        ComponentExecution {
            tasks: JoinSet::new(),
            component: Arc::new(component),
            stop_component: None,
            channels,
        }
    }

    pub fn start(&mut self) {
        let metadata: ComponentMetadata = self.component.metadata.clone();
        let shutdown: Arc<StopComponent> = Default::default();

        // Store shutdown state before we start the component properly
        let _ = self.stop_component.insert(shutdown.clone());

        match self.component.schedule {
            // Allow the component to manage its own scheduling
            Schedule::Unbounded { concurrency } => {
                for _ in 0..concurrency {
                    let environment: ExecutionEnvironment = ExecutionEnvironment::new(
                        metadata.clone(),
                        self.channels.clone(),
                        shutdown.clone(),
                    );

                    self.schedule_component(environment, None);
                }
            }
            // Schedule the component at set intervals
            Schedule::Interval { period_millis } => {
                let mut interval: Interval = interval(Duration::from_millis(period_millis));
                // Don't try and catch up with missed ticks
                interval.set_missed_tick_behavior(Delay);

                let environment: ExecutionEnvironment = ExecutionEnvironment::new(
                    metadata.clone(),
                    self.channels.clone(),
                    shutdown.clone(),
                );

                self.schedule_component(environment, Some(interval));
            }
        };
    }

    pub async fn stop(&mut self) {
        if let Some(stop_component) = &self.stop_component {
            stop_component.stop(self.tasks.len())
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.stop_component {
            None => true,
            Some(_) => false
        }
    }

    pub async fn kill(&mut self) {
        if self.tasks.len() > 0 {
            self.tasks.shutdown().await
        }
    }

    fn schedule_component(
        &mut self,
        mut environment: ExecutionEnvironment,
        mut interval: Option<Interval>,
    ) {
        let shutdown: Arc<StopComponent> = self.stop_component.clone().unwrap();
        let implementation: Arc<dyn Process> = self.component.implementation.clone();

        self.tasks.spawn(async move {
            loop {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }

                // Ensure that producers are shutdown properly
                if shutdown.is_stopped() {
                    break;
                }

                if let Err(err) = implementation.process(&mut environment).await {
                    match err {
                        ComponentError::ComponentShutdown => {
                            // TODO re-queue item in this case
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
