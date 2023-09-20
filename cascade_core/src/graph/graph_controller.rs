use std::collections::HashMap;
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
use crate::graph::CascadeGraph;
use crate::registry::ComponentRegistry;

type ConnectionsLock<'a> = RwLockWriteGuard<'a, HashMap<EdgeIndex, Arc<Connection>>>;

struct ComponentExecution {
    // Set to request shutdown
    shutdown: Arc<AtomicBool>,
    environment: Arc<ExecutionEnvironment>,
    // Active tasks for this execution
    handles: Vec<JoinHandle<()>>,
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

        let component: Arc<Component> = Arc::new(component);
        // Flag to indicate shutdown is requested
        let shutdown: Arc<AtomicBool> = Arc::new(Default::default());
        // Execution env abstracts recv and sending of items
        let environment: Arc<ExecutionEnvironment> = Arc::new(ExecutionEnvironment::new(
            metadata.clone(),
            component_channels.rx,
            component_channels.tx_named,
        ));

        // Create a new execution for this component
        let component_execution: ComponentExecution = ComponentExecution {
            environment: environment.clone(),
            shutdown: shutdown.clone(),

            handles: match component.schedule {
                // Allow the component to manage it's own scheduling
                Schedule::Unbounded { concurrency } => (0..concurrency)
                    .into_iter()
                    .map(|_| {
                        let component: Arc<Component> = Arc::clone(&component);
                        let environment: Arc<ExecutionEnvironment> = Arc::clone(&environment);
                        let shutdown: Arc<AtomicBool> = Arc::clone(&shutdown);

                        tokio::spawn(async move {
                            loop {
                                if shutdown.load(Ordering::Relaxed) {
                                    break;
                                }

                                Self::run_component(component.clone(), environment.clone()).await;
                            }
                        })
                    })
                    .collect(),
                // Schedule the component at set intervals
                Schedule::Interval { period_millis } => {
                    vec![tokio::spawn(async move {
                        let mut interval: Interval = interval(Duration::from_millis(period_millis));
                        // Don't try and catch up with missed ticks
                        interval.set_missed_tick_behavior(Delay);

                        // Create a task to run the component in
                        loop {
                            if shutdown.load(Ordering::Relaxed) {
                                break;
                            }

                            interval.tick().await;

                            Self::run_component(component.clone(), environment.clone()).await;
                        }
                    })]
                }
            },
        };

        self.active_executions.insert(node_idx, component_execution);

        Ok(metadata)
    }

    async fn run_component(component: Arc<Component>, execution: Arc<ExecutionEnvironment>) {
        let metadata: &ComponentMetadata = &component.metadata;

        match component.implementation.process(execution).await {
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

        // Indicate that the executions should stop
        execution.shutdown.store(true, Ordering::Relaxed);
        // Wait for it to stop

        // join!(&execution.handles).await;
        // .map_err(|_| StopComponentError::FailedToStop)?;

        let environment: Arc<ExecutionEnvironment> = execution.environment;

        Ok(environment.metadata.clone())
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
