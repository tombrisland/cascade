use std::iter::Map;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use petgraph::{Direction, EdgeDirection, Graph};
use petgraph::graph::NodeIndex;

use Component::{Processor, Producer};

use crate::{Item, Process, Produce};
use crate::connection::{Connection};

// Represents a graph of the entire flow
pub struct Flow {
    pub graph: Graph<Component, Connection>,

    // Producer indices
    producer_indices: Vec<NodeIndex>,

    // Last node added
    last_index: Option<NodeIndex>,
}

// Represent a single component in the flow
pub enum Component {
    Producer(Arc<dyn Produce>),
    Processor(Arc<dyn Process>),
}

fn handle_processor() {

}

pub fn run_flow(flow: &Flow) {
    for idx in flow.graph.node_indices() {
        let component = flow.graph.node_weight(idx).unwrap();

        match component {
            Producer(producer) => {
                handle_producer(flow, idx, producer);
            }
            Processor(processor) => {
                let outgoing = flow.retrieve_connections(idx, Direction::Outgoing);
                let incoming = flow.retrieve_connections(idx, Direction::Incoming);

                for conn in incoming {
                    let cloned = Arc::clone(processor);

                    let outgoing_connections: Vec<&Connection> = outgoing.clone();

                    thread::spawn(move || {
                        for input in conn.rx.lock().unwrap().iter() {
                            println!("Received item to from outgoing connection {}", idx.index());

                            let result = cloned.process(input).unwrap();

                            for outgoing_conn in outgoing_connections.iter() {
                                outgoing_conn.tx.send(result.clone());
                            }
                        }
                    });
                }
            }
        }
    }
}

fn handle_producer(flow: &Flow, component: NodeIndex, producer: &Arc<dyn Produce>) {
    let outgoing = flow.retrieve_connections(component, Direction::Outgoing);
    let cloned = Arc::clone(producer);

    thread::spawn(move || {
        loop {
            let item = cloned.produce().unwrap();

            for conn in outgoing {
                println!("Sending item to outgoing connection {}", component.index());
                conn.tx().send(item.clone());
            }

            thread::sleep(Duration::from_secs(5));
        }
    });
}

impl Flow {
    pub fn new() -> Flow {
        Flow {
            graph: Graph::new(),
            producer_indices: vec![],
            last_index: None,
        }
    }

    pub fn add_producer(mut self, producer: Arc<dyn Produce>) -> Flow {
        let index = self.graph.add_node(Producer(producer));

        self.producer_indices.push(index);
        self.last_index = Some(index);

        self
    }

    pub fn connect_to_previous(mut self, processor: Arc<dyn Process>) -> Flow {
        if self.last_index.is_some() {
            let destination = self.graph.add_node(Processor(processor));

            self.graph.add_edge(self.last_index.unwrap(), destination, Connection::Memory(Connection::new()));

            self.last_index = Some(destination);
        }

        self
    }

    fn retrieve_connections(&self, component: NodeIndex, direction: Direction) -> Vec<&Connection> {
        self.graph
            .edges_directed(component, direction)
            .map(|edge| { edge.weight().clone() }).collect()
    }
}