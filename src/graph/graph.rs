use petgraph::{Direction, Graph, Incoming, Outgoing};
use petgraph::graph::{EdgeIndex, NodeIndex, NodeIndices};
use petgraph::visit::EdgeRef;

use crate::component::definition::ComponentDefinition;
use crate::connection::definition::ConnectionDefinition;

pub type GraphInternal = Graph<ComponentDefinition, ConnectionDefinition>;

// Represents a graph of the entire graph
#[derive(Clone)]
pub struct CascadeGraph {
    pub graph_internal: GraphInternal,
}

impl CascadeGraph {
    pub fn get_node_indices(&self) -> NodeIndices {
        self.graph_internal.node_indices()
    }

    pub fn get_component_for_node(&self, node_idx: NodeIndex) -> Option<&ComponentDefinition> {
        self.graph_internal.node_weight(node_idx)
    }

    pub fn get_connection_for_edge(&self, edge_idx: EdgeIndex) -> Option<&ConnectionDefinition> {
        self.graph_internal.edge_weight(edge_idx)
    }

    pub fn get_incoming_for_node(&self, node_idx: NodeIndex) -> Vec<EdgeIndex> {
        self.get_connections_directed(node_idx, Incoming)
    }

    pub fn get_outgoing_for_node(&self, node_idx: NodeIndex) -> Vec<EdgeIndex> {
        self.get_connections_directed(node_idx, Outgoing)
    }

    pub fn get_edges_for_node(&self, node_idx: NodeIndex) -> Vec<EdgeIndex> {
        self.graph_internal
            .edges(node_idx)
            .map(|edge| edge.id() as EdgeIndex)
            .collect()
    }

    fn get_connections_directed(
        &self,
        node_idx: NodeIndex,
        direction: Direction,
    ) -> Vec<EdgeIndex> {
        self.graph_internal
            .edges_directed(node_idx, direction)
            .map(|edge| edge.id() as EdgeIndex)
            .collect()
    }
}
