use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use petgraph::graph::{EdgeIndex, NodeIndex};
use tokio::sync::{OwnedRwLockWriteGuard, RwLock};

use crate::component::component::Component;
use crate::component::definition::ComponentDefinition;
use crate::component::execution::ExecutionEnvironment;
use crate::component::Process;
use crate::component_registry::ComponentRegistry;
use crate::connection::{ConnectionRead, ConnectionWrite, create_connection, IncomingStream};
use crate::connection::definition::ConnectionDefinition;
use crate::graph::error::StartComponentError;
use crate::graph::graph::CascadeGraph;

#[derive(Debug)]
pub struct Connection {
    read: Arc<RwLock<ConnectionRead>>,
    write: Arc<ConnectionWrite>
}

// Control the execution of a FlowGraph
pub struct CascadeController {
    pub component_registry: ComponentRegistry,

    pub graph_definition: CascadeGraph,

    pub connections: Arc<RwLock<HashMap<EdgeIndex, Connection>>>,
}

impl CascadeController {
    pub fn new(component_registry: ComponentRegistry) -> CascadeController {
        CascadeController {
            graph_definition: CascadeGraph {
                graph_internal: Default::default(),
            },
            component_registry,

            connections: Default::default(),
        }
    }

    pub async fn init_channels(&mut self, idx: &EdgeIndex) {
        let def: &ConnectionDefinition = self
            .graph_definition
            .get_connection_for_edge(idx.clone())
            .unwrap();

        if !self.connections.read().await.contains_key(idx) {
            let (read, write) = create_connection(def);

            let connection = Connection {
                read: Arc::new(RwLock::new(read)),
                write: Arc::new(write),
            };

            self.connections.write().await.insert(idx.clone(), connection);
        }
    }

    pub async fn start_component(
        &mut self,
        node_idx: NodeIndex,
    ) -> Result<(), StartComponentError> {
        for edge_idx in self.graph_definition.get_edges_for_node(node_idx) {
            self.init_channels(&edge_idx).await;
        }

        let graph: &CascadeGraph = &self.graph_definition;

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
            "Starting {:?};{} with id {}",
            def.component_type, def.type_name, component.metadata.id
        );

        let implementation: Arc<dyn Process> = component.implementation;

        let graph: CascadeGraph = graph.clone();

        let connections : Arc<RwLock<HashMap<EdgeIndex, Connection>>> = Arc::clone(&self.connections);

        let connections_lock = connections.read().await;

        let mut connection_writes: Vec<Arc<ConnectionWrite>> = vec![];

        for edge_idx in graph.get_outgoing_for_node(node_idx) {
            let connection: &Connection = connections_lock.get(&edge_idx).unwrap();

            connection_writes.push(Arc::clone(&connection.write))
        }

        let mut connection_read_locks: Vec<OwnedRwLockWriteGuard<ConnectionRead>> = vec![];

        for edge_idx in graph.get_incoming_for_node(node_idx) {
            let connection: &Connection = connections_lock.get(&edge_idx).unwrap();

            connection_read_locks.push(connection.read.clone().write_owned().await);
        }

        let incoming_stream: IncomingStream =
            ConnectionRead::create_incoming_stream(connection_read_locks).await;

        let mut execution = ExecutionEnvironment::new(incoming_stream, connection_writes);

        tokio::spawn(async move {

            loop {
                implementation.process(&mut execution).await.unwrap();
            }
        });

        Ok(())
    }
}
