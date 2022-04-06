use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use petgraph::graph::NodeIndex;
use petgraph::Graph;

use crate::flow::builder::FlowComponent;
use crate::flow::connection::FlowConnection;
use crate::flow::item::FlowItem;

pub mod builder;
mod configuration;
pub mod connection;
pub mod controller;
pub mod item;

// Represents a graph of the entire flow
pub struct FlowGraph {
    pub graph: Arc<Graph<FlowComponent, FlowConnection>>,

    producer_indices: Vec<NodeIndex>,
}

enum ExecutionStatus {
    Incomplete,
    Complete,
    Errored,
}

// Describe an individual item's path through the flow
struct FlowExecution<'a> {
    graph: &'a Graph<FlowComponent, FlowConnection>,
    // Map of node index to join handle?
    component_status: Arc<Mutex<HashMap<usize, ExecutionStatus>>>,
    // From this we can control waiting on different nodes before

    // How do we merge results back together?

    // Instead of taking a singular FlowItem we bring back ExecutionSession / FlowSession
    // This can contain a varargs of FlowItems, we can control the behaviour of these to either be just a list of items OR
    // Merged items from a previous branch -> this can be controlled by parentId which should be set only by FlowItem.child()
    // We need something to detect a branch and st
}

impl FlowExecution<'_> {
    fn new(graph: &Graph<FlowComponent, FlowConnection>) -> FlowExecution {
        FlowExecution {
            graph,
            component_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

struct ExecutionSession {
    items: Vec<FlowItem>,
}

impl ExecutionSession {}
