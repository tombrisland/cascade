use std::collections::HashMap;
use std::sync::Arc;

use petgraph::{Direction, Graph, Incoming, Outgoing};
use petgraph::graph::{EdgeIndex, NodeIndex, NodeIndices};
use petgraph::visit::EdgeRef;

use crate::component::{ComponentDefinition, ComponentInstance, ComponentMetadata, ComponentType};
use crate::component_registry::registry::ComponentRegistry;
use crate::connection::{Connection, ConnectionDefinition, Connections};
use crate::processor::{Process, Processor};
use crate::producer::{Produce, Producer};

pub type GraphInternal = Graph<ComponentDefinition, ConnectionDefinition>;

#[derive(Clone)]
pub struct ConnectionDetails {
    pub incoming: Connections,
    pub outgoing: Connections,
}

// Represents a graph of the entire graph
#[derive(Clone)]
pub struct CascadeGraph {
    pub(crate) component_registry: ComponentRegistry,

    pub graph_internal: GraphInternal,
    pub edge_state: HashMap<EdgeIndex, Arc<Connection>>,
}

impl CascadeGraph {
    pub fn get_node_indices(&self) -> NodeIndices {
        self.graph_internal.node_indices()
    }

    pub fn get_component_for_idx(&self, node_idx: NodeIndex) -> Option<(ComponentInstance, &ComponentMetadata)> {
        let def: &ComponentDefinition = self.graph_internal.node_weight(node_idx)?;
        let metadata: ComponentMetadata = def.metadata.clone();

        let instance: ComponentInstance = match def.component_type {
            ComponentType::Producer => {
                let produce: Arc<dyn Produce> = self.component_registry
                    .get_producer(&metadata.type_name, def.config.clone())?;

                ComponentInstance::Producer(Producer::new(metadata, produce))
            }
            ComponentType::Processor => {
                let process: Arc<dyn Process> = self.component_registry
                    .get_processor(&metadata.type_name, def.config.clone())?;

                ComponentInstance::Processor(Processor::new(metadata, process))
            }
        };

        // Return metadata reference
        Some((instance, &def.metadata))
    }

    pub fn find_or_create(&mut self, conn_def: &(EdgeIndex, ConnectionDefinition)) -> Arc<Connection> {
        let (idx, def) = conn_def;

        if !self.edge_state.contains_key(&idx) {
            let connection = Arc::new(Connection::new(def.clone()));

            self.edge_state.insert(idx.clone(), connection);
        }

        self.edge_state.get(&idx).unwrap().clone()
    }

    pub fn get_connections(&mut self, node_idx: NodeIndex) -> ConnectionDetails {
        let incoming_def: Vec<(EdgeIndex, ConnectionDefinition)> = self.get_incoming(node_idx);
        let outgoing_def: Vec<(EdgeIndex, ConnectionDefinition)> = self.get_outgoing(node_idx);

        let incoming: Vec<Arc<Connection>> = incoming_def.iter().map(|conn_def| self.find_or_create(conn_def)).collect();
        let outgoing: Vec<Arc<Connection>> = outgoing_def.iter().map(|conn_def| self.find_or_create(conn_def)).collect();

        ConnectionDetails {
            incoming: Connections::new(incoming),
            outgoing: Connections::new(outgoing),
        }
    }

    pub fn get_incoming(&self, node_idx: NodeIndex) -> Vec<(EdgeIndex, ConnectionDefinition)> {
        self.get_connections_directed(node_idx, Incoming)
    }

    pub fn get_outgoing(&self, node_idx: NodeIndex) -> Vec<(EdgeIndex, ConnectionDefinition)> {
        self.get_connections_directed(node_idx, Outgoing)
    }

    fn get_connections_directed(&self, node_idx: NodeIndex, direction: Direction) -> Vec<(EdgeIndex, ConnectionDefinition)> {
        self.graph_internal.edges_directed(node_idx, direction)
            .map(|edge| (edge.id() as EdgeIndex, edge.weight().clone())).collect()
    }
}
