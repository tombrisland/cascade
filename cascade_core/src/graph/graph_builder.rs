use petgraph::Graph;
use petgraph::graph::NodeIndex;
use cascade_api::component::definition::ComponentDefinition;
use cascade_api::connection::definition::ConnectionDefinition;
use crate::graph::{CascadeGraph, GraphInternal};


// Represents a graph of the entire graph
pub struct CascadeGraphBuilder {
    graph_internal: GraphInternal,

    // Last node added
    last_index: Option<NodeIndex>,
}

impl CascadeGraphBuilder {
    pub fn new() -> CascadeGraphBuilder {
        CascadeGraphBuilder {
            graph_internal: Graph::new(),
            last_index: None,
        }
    }

    // Add component with no outgoing connections
    pub fn add_component(
        mut self,
        def: ComponentDefinition,
    ) -> CascadeGraphBuilder {
        let index = self.graph_internal.add_node(def);

        // Set the last_index to enable connect_to_previous
        self.last_index = Some(index);

        self
    }

    // Add a component connected to the previous component
    pub fn connect_to_previous(
        &mut self,
        def: ComponentDefinition,
    ) -> &CascadeGraphBuilder {
        if self.last_index.is_some() {
            let source: NodeIndex = self.last_index.unwrap();
            let destination: NodeIndex = self.graph_internal.add_node(def);

            self.graph_internal.add_edge(
                source.clone(),
                destination,
                ConnectionDefinition::new(source.index(), destination.index()),
            );

            self.last_index = Some(destination);
        }

        self
    }

    pub fn build(self) -> CascadeGraph {
        CascadeGraph {
            graph_internal: self.graph_internal,
        }
    }
}
