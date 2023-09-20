use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender};
use log::{error, info};
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::Direction;
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior::Delay;
use tokio::time::{interval, Interval};

use cascade_component::component::{Component, ComponentMetadata, Schedule};
use cascade_component::definition::ComponentDefinition;
use cascade_component::execution_env::ExecutionEnvironment;
use cascade_connection::definition::ConnectionDefinition;
use cascade_connection::Connection;
use cascade_payload::CascadeItem;

use crate::graph::graph_controller_error::{StartComponentError, StopComponentError};
use crate::graph::{CascadeGraph};
use crate::registry::ComponentRegistry;

type ConnectionsLock<'a> = RwLockWriteGuard<'a, HashMap<EdgeIndex, Arc<Connection>>>;

struct ComponentExecution {
    handle: JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
    environment: Arc<Mutex<ExecutionEnvironment>>,
}

pub struct CascadeController {
    pub component_registry: ComponentRegistry,

    pub graph_definition: Arc<Mutex<CascadeGraph>>,

    active_executions: HashMap<NodeIndex, ComponentExecution>,
    connections: Arc<RwLock<HashMap<EdgeIndex, Arc<Connection>>>>,
}

impl CascadeController {
    pub fn new(component_registry: ComponentRegistry) -> CascadeController {
        CascadeController {
            graph_definition: Arc::new(Mutex::new(CascadeGraph {
                graph_internal: Default::default(),
            })),
            component_registry,

            connections: Default::default(),
            active_executions: Default::default(),
        }
    }

    pub async fn start_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<ComponentMetadata, StartComponentError> {
        let graph: MutexGuard<CascadeGraph> = self.graph_definition.lock().await;
        let connections_lock: ConnectionsLock = self.connections.write().await;

        // Initialise any missing connections and return all relevant references
        let component_channels: ComponentChannels =
            init_channels_for_node(&graph, connections_lock, node_idx);

        // Fail if the index isn't present in the graph
        let def: &ComponentDefinition = graph
            .get_component_for_node(node_idx)
            .ok_or(StartComponentError::InvalidNodeIndex(node_idx.index()))?;

        // Fail if the component impl type isn't in the registry
        let component: Component = self
            .component_registry
            .get_component(def)
            .ok_or(StartComponentError::MissingComponent(def.type_name.clone()))?;
        let metadata: ComponentMetadata = component.metadata.clone();

        info!("{} is starting", metadata);

        // Execution env abstracts recv and sending of items
        let environment: Arc<Mutex<ExecutionEnvironment>> = Arc::new(Mutex::new(
            ExecutionEnvironment::new(metadata.clone(), component_channels.rx, component_channels.tx_named).await,
        ));

        let shutdown: Arc<AtomicBool> = Arc::new(Default::default());

        let component_execution: ComponentExecution = ComponentExecution {
            environment: environment.clone(),
            shutdown: shutdown.clone(),
            // Create a new task to run the execution within
            handle: tokio::spawn(async move {
                let execution: Arc<Mutex<ExecutionEnvironment>> = Arc::clone(&environment);

                match component.schedule {
                    // Allow the component to manage it's own scheduling
                    Schedule::Unbounded => loop {
                        if shutdown.load(Ordering::Relaxed) {
                            break;
                        }

                        Self::run_component(&component, execution.clone()).await;
                    },
                    // Schedule the component at set intervals
                    Schedule::Interval { period_millis } => {
                        let mut interval: Interval = interval(Duration::from_millis(period_millis));
                        // Don't try and catch up with missed ticks
                        interval.set_missed_tick_behavior(Delay);

                        // Create a task to run the component in
                        loop {
                            if shutdown.load(Ordering::Relaxed) {
                                break;
                            }

                            interval.tick().await;

                            Self::run_component(&component, execution.clone()).await;
                        }
                    }
                };
            }),
        };

        self.active_executions.insert(node_idx, component_execution);

        Ok(metadata)
    }

    async fn run_component<'a>(component: &Component, execution: Arc<Mutex<ExecutionEnvironment>>) {
        let metadata: &ComponentMetadata = &component.metadata;
        // Lock the execution environment for the duration of processing
        let mut execution_lock: MutexGuard<ExecutionEnvironment> = execution.lock().await;

        match component
            .implementation
            .process(execution_lock.deref_mut())
            .await
        {
            Ok(_) => {}
            Err(err) => {
                error!("{} failed with {:?}", metadata, err)
            }
        }
    }

    pub async fn stop_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<ComponentMetadata, StopComponentError> {
        // Try and find a relevant execution
        let execution: ComponentExecution = self
            .active_executions
            .remove(&node_idx)
            // Error if we there is no execution started
            .ok_or(StopComponentError::ComponentNotStarted(node_idx.index()))?;

        // Indicate that the execution should stop
        execution.shutdown.store(true, Ordering::Relaxed);
        // Wait for it to stop
        let _ = &execution
            .handle
            .await
            .map_err(|_| StopComponentError::FailedToStop)?;

        let environment: Arc<Mutex<ExecutionEnvironment>> = execution.environment.clone();

        // We can lock the execution environment now that the task is stopped
        let environment_lock: MutexGuard<ExecutionEnvironment> = environment.lock().await;

        Ok(environment_lock.metadata.clone())
    }
}

pub struct ComponentChannels {
    rx: Vec<Arc<Receiver<CascadeItem>>>,
    tx_named: HashMap<String, Arc<Sender<CascadeItem>>>,
}

fn init_channels_for_node(
    graph: &MutexGuard<CascadeGraph>,
    mut connections_lock: ConnectionsLock,
    node_idx: NodeIndex,
) -> ComponentChannels {
    let mut rx: Vec<Arc<Receiver<CascadeItem>>> = vec![];
    let mut tx_named: HashMap<String, Arc<Sender<CascadeItem>>> = Default::default();

    for (direction, idx) in graph.get_edges_for_node(node_idx) {
        let def: &ConnectionDefinition = graph.get_connection_for_edge(idx.clone()).unwrap();

        // Insert the new connection into the map for sharing across components
        let connection: Arc<Connection> = Arc::clone(
            connections_lock
                .entry(idx.clone())
                .or_insert_with(|| Arc::new(Connection::new(idx.index(), def))),
        );

        match direction {
            Direction::Outgoing => {
                // Include entry in the map by name
                tx_named.insert(connection.name.clone(), connection.tx.clone());
            }
            Direction::Incoming => rx.push(connection.rx.clone()),
        };
    }

    // Return channels relevant to this node
    ComponentChannels { rx, tx_named }
}
