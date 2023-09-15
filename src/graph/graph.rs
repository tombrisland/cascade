use std::sync::Arc;

use petgraph::{Direction, Graph, Incoming, Outgoing};
use petgraph::graph::{NodeIndex, NodeIndices};

use crate::connection::{ComponentOutput, ConnectionEdge};
use crate::graph::graph_builder::ComponentNode;

pub type GraphInternal = Graph<ComponentNode, Arc<ConnectionEdge>>;

// Represents a graph of the entire graph
pub struct CascadeGraph {
    pub graph_internal: GraphInternal,
}

impl CascadeGraph {
    pub fn get_nodes(&self) -> NodeIndices {
        self.graph_internal.node_indices()
    }

    pub fn get_node_component(&self, node_idx: NodeIndex) -> Option<&ComponentNode> {
        self.graph_internal.node_weight(node_idx)
    }
    pub fn get_incoming_connection(&self, node_idx: NodeIndex) -> Arc<ConnectionEdge> {
        let connections = self.get_connections_directed(node_idx, Incoming);

        connections.get(0).unwrap().clone()
    }

    pub fn get_output(&self, node_idx: NodeIndex) -> ComponentOutput {
        let connections: Vec<Arc<ConnectionEdge>> = self.get_connections_directed(node_idx, Outgoing);

        ComponentOutput::new(connections)
    }

    fn get_connections_directed(&self, node_idx: NodeIndex, direction: Direction) -> Vec<Arc<ConnectionEdge>> {
        self.graph_internal.edges_directed(node_idx, direction).map(|edge| edge.weight().clone()).collect()
    }
}