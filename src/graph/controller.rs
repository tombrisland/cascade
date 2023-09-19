use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::{error, info};
use petgraph::Direction;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time::{interval, Interval};
use tokio::time::MissedTickBehavior::Delay;

use crate::component::component::{Component, ComponentMetadata, Schedule};
use crate::component::definition::ComponentDefinition;
use crate::component::execution::ExecutionEnvironment;
use crate::component_registry::ComponentRegistry;
use crate::connection::{Connection, DirectedConnections};
use crate::connection::definition::ConnectionDefinition;
use crate::graph::error::{StartComponentError, StopComponentError};
use crate::graph::graph::CascadeGraph;
use crate::graph::item::CascadeItem;

type ConnectionsLock<'a> = RwLockWriteGuard<'a, HashMap<EdgeIndex, Arc<Connection>>>;

struct ComponentExecution {
    handle: JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
    environment: Arc<Mutex<ExecutionEnvironment>>,
}

pub struct CascadeController {
    pub component_registry: ComponentRegistry,

    pub graph_definition: Arc<Mutex<CascadeGraph>>,

    component_executions: HashMap<NodeIndex, ComponentExecution>,
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
            component_executions: Default::default(),
        }
    }

    pub async fn start_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<(), StartComponentError> {
        let graph: MutexGuard<CascadeGraph> = self.graph_definition.lock().await;
        let connections_lock: ConnectionsLock = self.connections.write().await;

        // Initialise any missing connections and return all relevant references
        let directed_connections: DirectedConnections =
            init_connections_for_node(&graph, connections_lock, node_idx);

        // Fail if the index isn't present in the graph
        let def: &ComponentDefinition = graph
            .get_component_for_node(node_idx)
            .ok_or(StartComponentError::InvalidNodeIndex(node_idx.index()))?;

        // Fail if the component impl type isn't in the registry
        let component: Component = self
            .component_registry
            .get_component(def)
            .ok_or(StartComponentError::MissingComponent(def.type_name.clone()))?;

        info!("{} is starting", component.metadata);

        // Execution env abstracts recv and sending of items
        let environment: Arc<Mutex<ExecutionEnvironment>> = Arc::new(Mutex::new(
            ExecutionEnvironment::new(component.metadata.clone(), directed_connections).await,
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

        self.component_executions
            .insert(node_idx, component_execution);

        Ok(())
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

    pub async fn stop_component(&mut self, node_idx: NodeIndex) -> Result<(), StopComponentError> {
        // Try and find a relevant execution
        let execution: ComponentExecution = self
            .component_executions
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

        // Lock the execution environment now that the task is stopped
        let connections_lock: ConnectionsLock = self.connections.write().await;
        let mut environment_lock: MutexGuard<ExecutionEnvironment> = environment.lock().await;

        // Take back ownership of the receivers and store in connections
        let receivers: Vec<Receiver<CascadeItem>> = environment_lock.take_receivers().unwrap();

        for (idx, receiver) in receivers.into_iter().enumerate() {
            let edge_idx: EdgeIndex =
                EdgeIndex::new(environment_lock.receiver_indexes.get(idx).unwrap().clone());

            let mut connection_lock: RwLockWriteGuard<Option<Receiver<CascadeItem>>> =
                connections_lock.get(&edge_idx).unwrap().rx.write().await;

            // Store the receiver back in the original connection
            connection_lock.replace(receiver);
        }

        Ok(())
    }
}

fn init_connections_for_node<'a>(
    graph: &MutexGuard<CascadeGraph>,
    mut connections_lock: ConnectionsLock,
    node_idx: NodeIndex,
) -> DirectedConnections {
    let edge_indices: Vec<(EdgeIndex, Direction)> = graph.get_edges_for_node(node_idx);

    DirectedConnections::new(
        edge_indices
            .iter()
            .map(|(edge_idx, direction)| {
                let def: &ConnectionDefinition =
                    graph.get_connection_for_edge(edge_idx.clone()).unwrap();

                // Build a map of Direction to Connections
                let connection: Arc<Connection> = Arc::clone(
                    connections_lock
                        .entry(edge_idx.clone())
                        .or_insert_with(|| Arc::new(Connection::new(edge_idx, def))),
                );

                (direction.clone(), connection)
            })
            .collect(),
    )
}
