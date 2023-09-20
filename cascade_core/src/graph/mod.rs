use petgraph::{Direction, Graph, Incoming, Outgoing};
use petgraph::graph::{EdgeIndex, NodeIndex, NodeIndices};
use petgraph::visit::EdgeRef;

use cascade_component::definition::ComponentDefinition;
use cascade_connection::definition::ConnectionDefinition;

pub mod graph_builder;
pub mod graph_controller;
pub mod graph_controller_error;

pub type GraphInternal = Graph<ComponentDefinition, ConnectionDefinition>;

// Represents a graph of the entire graph
// TODO remove this clone
#[derive(Clone)]
pub struct CascadeGraph {
    pub graph_internal: GraphInternal,
}

impl CascadeGraph {
    pub fn _get_node_indices(&self) -> NodeIndices {
        self.graph_internal.node_indices()
    }

    pub fn get_component_for_node(&self, node_idx: NodeIndex) -> Option<&ComponentDefinition> {
        self.graph_internal.node_weight(node_idx)
    }

    pub fn get_connection_for_edge(&self, edge_idx: EdgeIndex) -> Option<&ConnectionDefinition> {
        self.graph_internal.edge_weight(edge_idx)
    }

    pub fn get_edges_for_node(&self, node_idx: NodeIndex) -> Vec<(Direction, EdgeIndex)> {
        self.graph_internal
            .edges_directed(node_idx, Incoming)
            // The existing .edges impl only returns Outgoing edges
            .chain(self.graph_internal.edges_directed(node_idx, Outgoing))
            .map(|edge| {
                (
                    // Include direction in tuple
                    if edge.source() == node_idx {
                        Outgoing
                    } else {
                        Incoming
                    },
                    edge.id(),
                )
            })
            .collect()
    }
}
