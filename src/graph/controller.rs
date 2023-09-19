use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use petgraph::Direction;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};
use tokio::task::JoinHandle;

use crate::component::component::Component;
use crate::component::definition::ComponentDefinition;
use crate::component::execution::ExecutionEnvironment;
use crate::component::Process;
use crate::component_registry::ComponentRegistry;
use crate::connection::{Connection, DirectedConnections};
use crate::connection::definition::ConnectionDefinition;
use crate::graph::error::StartComponentError;
use crate::graph::graph::CascadeGraph;

type ConnectionsLock<'a> = RwLockWriteGuard<'a, HashMap<EdgeIndex, Arc<Connection>>>;

pub struct CascadeController {
    pub component_registry: ComponentRegistry,

    pub graph_definition: Arc<Mutex<CascadeGraph>>,

    pub component_handles: HashMap<NodeIndex, JoinHandle<()>>,
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
            component_handles: Default::default(),
        }
    }

    pub async fn start_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<(), StartComponentError> {
        let graph: MutexGuard<CascadeGraph> = self.graph_definition.lock().await;
        let connections_lock: ConnectionsLock = self.connections.write().await;

        // Initialise any missing connections and return all relevant references
        let directed_connections: DirectedConnections = init_connections_for_node(&graph, connections_lock, node_idx);

        // Fail if the index isn't present in the graph
        let def: &ComponentDefinition = graph
            .get_component_for_node(node_idx)
            .ok_or(StartComponentError::InvalidNodeIndex(node_idx.index()))?;

        // Fail if the component impl type isn't in the registry
        let component: Component = self
            .component_registry
            .get_component(def)
            .ok_or(StartComponentError::MissingComponent(def.type_name.clone()))?;

        info!(
            "[{:?}:{}] starting with id {}",
            def.component_type, def.type_name, component.metadata.id
        );

        let implementation: Arc<dyn Process> = component.implementation;

        let mut execution =
            ExecutionEnvironment::new(directed_connections).await;

        let handle: JoinHandle<()> = tokio::spawn(async move {
            loop {
                implementation.process(&mut execution).await.unwrap();
            }
        });

        self.component_handles.insert(node_idx, handle);

        Ok(())
    }
}


pub fn init_connections_for_node<'a>(
    graph: &MutexGuard<CascadeGraph>,
    mut connections_lock: ConnectionsLock,
    node_idx: NodeIndex,
) -> DirectedConnections {
    let edge_indices: Vec<(EdgeIndex, Direction)> =
        graph.get_edges_for_node(node_idx);

    DirectedConnections::new(
        edge_indices
            .iter()
            .map(|(edge_idx, direction)| {
                let def: &ConnectionDefinition = graph
                    .get_connection_for_edge(edge_idx.clone())
                    .unwrap();

                let connection: Arc<Connection> = Arc::clone(
                    connections_lock
                        .entry(edge_idx.clone())
                        .or_insert_with(|| Arc::new(Connection::new(def))),
                );

                (direction.clone(), connection)
            })
            .collect(),
    )
}
