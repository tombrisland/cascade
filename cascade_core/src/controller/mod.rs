use std::collections::HashMap;
use std::sync::Arc;

use async_channel::{Receiver, Sender, unbounded};
use log::info;
use petgraph::Direction;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{RwLock, RwLockWriteGuard};

use cascade_api::component::component::{Component, ComponentMetadata, Schedule};
use cascade_api::component::definition::ComponentDefinition;
use cascade_api::connection::{ComponentChannels, Connection};
use cascade_api::connection::definition::ConnectionDefinition;
use cascade_api::message::InternalMessage;

use crate::controller::error::{RemoveConnectionError, StartComponentError, StopComponentError};
use crate::controller::execution::ComponentExecution;
use crate::graph::CascadeGraph;
use crate::registry::ComponentRegistry;

pub mod error;
mod execution;

pub type ConnectionsMap = HashMap<EdgeIndex, Arc<Connection>>;

pub struct CascadeController {
    pub component_registry: ComponentRegistry,

    pub graph_definition: Arc<RwLock<CascadeGraph>>,

    pub active_executions: HashMap<NodeIndex, ComponentExecution>,
    pub connections: Arc<RwLock<ConnectionsMap>>,
}

impl CascadeController {
    pub fn new(component_registry: ComponentRegistry) -> CascadeController {
        CascadeController {
            graph_definition: Arc::new(RwLock::new(CascadeGraph {
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
        let graph: RwLockWriteGuard<CascadeGraph> = self.graph_definition.write().await;
        let connections_lock: RwLockWriteGuard<ConnectionsMap> = self.connections.write().await;

        // Initialise any missing connections and return all relevant references
        let channels: ComponentChannels =
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
        let schedule: Schedule = component.schedule.clone();

        info!("{} is starting with schedule {:?}", metadata, schedule);

        let mut execution: ComponentExecution = ComponentExecution::new(component, channels);
        execution.start();
        self.active_executions.insert(node_idx, execution);

        Ok(metadata)
    }

    pub async fn stop_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<ComponentMetadata, StopComponentError> {
        // Try and find a relevant execution
        let mut execution: ComponentExecution = self
            .active_executions
            .remove(&node_idx)
            // Error if we there is no execution started
            .ok_or(StopComponentError::ComponentNotStarted(node_idx.index()))?;

        execution.stop().await.unwrap();

        Ok(execution.component.metadata.clone())
    }

    pub async fn remove_connection(
        &mut self,
        _edge_idx: EdgeIndex,
    ) -> Result<ComponentMetadata, RemoveConnectionError> {
        // Check if there are any active executions relating to the connection

        // // Try and find a relevant execution
        // let execution: ComponentExecution = self
        //     .active_executions
        //     .remove(&node_idx)
        //     // Error if we there is no execution started
        //     .ok_or(StopComponentError::ComponentNotStarted(node_idx.index()))?;
        //
        // // Indicate that the executions should stop
        // execution.shutdown.store(true, Ordering::Relaxed);
        // // Wait for it to stop
        //
        // // join!(&execution.handles).await;
        // // .map_err(|_| StopComponentError::FailedToStop)?;
        //
        // let environment: Arc<ExecutionEnvironment> = execution.environment;
        //
        // Ok(environment.metadata.clone())
        Err(RemoveConnectionError::ConnectionRunning(vec![]))
    }
}

fn init_channels_for_node(
    graph: &RwLockWriteGuard<CascadeGraph>,
    mut connections_lock: RwLockWriteGuard<ConnectionsMap>,
    node_idx: NodeIndex,
) -> ComponentChannels {
    // Receivers must be owned
    let mut rx_channels: Vec<Receiver<InternalMessage>> = Default::default();
    let mut tx_named: HashMap<String, Arc<Sender<InternalMessage>>> = Default::default();

    for (direction, idx) in graph.get_edges_for_node(node_idx) {
        let def: &ConnectionDefinition = graph.get_connection_for_edge(idx.clone()).unwrap();

        // Insert the new connection into the map for sharing across components
        let connection: Arc<Connection> = Arc::clone(
            connections_lock
                .entry(idx.clone())
                .or_insert_with(|| Arc::new(Connection::new(def))),
        );

        match direction {
            // Include entry in the map by name
            Direction::Outgoing => {
                tx_named.insert(connection.name.clone(), connection.tx.clone());
            }
            Direction::Incoming => rx_channels.push(connection.rx.write().unwrap().take().unwrap()),
        };
    }

    // Create an extra channel to send signals to the components
    let (tx_signal, rx_signal): (Sender<InternalMessage>, Receiver<InternalMessage>) = unbounded();

    rx_channels.push(rx_signal);

    ComponentChannels {
        rx: rx_channels,
        tx_signal: Arc::new(tx_signal),
        tx_named,
    }
}
