use crate::component::Component;
use crate::connection::ComponentChannels;
use crate::controller::environment::ExecutionEnvironment;
use cascade_api::component::component::{Schedule};
use cascade_api::component::error::ComponentError;
use cascade_api::component::Process;
use log::error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, MutexGuard};
use tokio::task;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::MissedTickBehavior::Delay;
use tokio::time::{interval, Interval};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct ComponentShutdown {
    token: CancellationToken,
    tasks: Option<Arc<Mutex<JoinSet<()>>>>,

    join_handle: Option<JoinHandle<()>>,
}

impl ComponentShutdown {
    /// Attempt to gracefully join tasks
    pub fn stop(&mut self, tasks: JoinSet<()>) {
        // Signal tasks to stop
        self.token.cancel();
        self.tasks = Some(Arc::new(Mutex::new(tasks)));

        let tasks: Option<Arc<Mutex<JoinSet<()>>>> = self.tasks.clone();

        // Try and join all active tasks
        let _ = self.join_handle.insert(task::spawn(async move {
            if let Some(tasks) = tasks {
                let mut guard: MutexGuard<JoinSet<()>> = tasks.lock().await;

                while !guard.is_empty() {
                    // TODO handle errors in some way - at least log them
                    guard.join_next().await;
                }
            }
        }));
    }

    /// Shutdown all associated tasks
    pub async fn kill(&mut self) {
        if let Some(tasks) = &self.tasks {
            let mut guard: MutexGuard<JoinSet<()>> = tasks.lock().await;

            guard.shutdown().await;
        }
    }
}

pub struct ComponentExecution {
    pub component: Arc<Component>,
    channels: ComponentChannels,

    // Active tasks for this execution
    tasks: Option<JoinSet<()>>,
    shutdown: ComponentShutdown,
}

impl ComponentExecution {
    pub fn new(component: Component, channels: ComponentChannels) -> ComponentExecution {
        ComponentExecution {
            tasks: Some(JoinSet::new()),
            component: Arc::new(component),
            channels,
            shutdown: Default::default(),
        }
    }

    pub fn start(&mut self) {
        // TODO throw if we try and start without expected configured channels

        match self.component.schedule {
            // Allow the component to manage its own scheduling
            Schedule::Unbounded { concurrency } => {
                for _ in 0..concurrency {
                    let environment: ExecutionEnvironment = ExecutionEnvironment::new(
                        self.channels.clone(),
                        self.shutdown.token.clone(),
                    );

                    self.schedule_component(environment, None);
                }
            }
            // Schedule the component at set intervals
            Schedule::Interval { period_millis } => {
                let mut interval: Interval = interval(Duration::from_millis(period_millis));
                // Don't try and catch up with missed ticks
                interval.set_missed_tick_behavior(Delay);

                let environment: ExecutionEnvironment =
                    ExecutionEnvironment::new(self.channels.clone(), self.shutdown.token.clone());

                self.schedule_component(environment, Some(interval));
            }
        };
    }

    pub async fn stop(&mut self) {
        if let Some(tasks) = self.tasks.take() {
            self.shutdown.stop(tasks);
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.shutdown.token.is_cancelled() && self.tasks.is_none()
    }

    pub async fn kill(&mut self) {
        // TODO this should rollback all sessions before attempting kill
        if self.shutdown.token.is_cancelled() && !self.tasks.is_none() {
            self.shutdown.kill().await;
        }
    }

    pub fn active_tasks(&self) -> Option<usize> {
        let active_tasks: &usize = &self.tasks.as_ref()?.len();

        Some(active_tasks.clone())
    }

    fn schedule_component(
        &mut self,
        mut environment: ExecutionEnvironment,
        mut interval: Option<Interval>,
    ) {
        let implementation: Arc<dyn Process> = self.component.implementation.clone();
        let tasks: &mut JoinSet<()> = self.tasks.get_or_insert(JoinSet::new());

        tasks.spawn(async move {
            loop {
                if let Some(interval) = interval.as_mut() {
                    interval.tick().await;
                }

                // TODO this should cause the execution to backoff - we need a separate interval for this
                // TODO actually maybe we could get a task to listen on the recv from the queue that matters

                // TODO should also move to a single receiver - multiple processor model - avoiding more spinning threads
                if !environment.tx_has_capacity() {
                    continue;
                }

                // Cancel execution if this component is stopped
                if environment.shutdown_token.is_cancelled() {
                    // Break loop and join task
                    break;
                }

                if let Err(err) = implementation.process(&mut environment).await {
                    // Rollback the session to the input queue
                    environment.rollback().await.unwrap();

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
