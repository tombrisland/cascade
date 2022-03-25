use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use petgraph::graph::NodeIndex;

use crate::flow::connection::ComponentConnections;
use crate::flow::FlowComponent::{Processor, Producer};
use crate::flow::item::FlowItem;
use crate::FlowGraph;
use crate::processor::Process;
use crate::producer::Produce;

// Control the execution of a FlowGraph
pub struct FlowController {
    pub flow: FlowGraph,

    // Map of indices to active threads
    // pub handles: Map<NodeIndex, JoinHandle<()>>
}

impl FlowController {
    // Start every component in the flow
    pub fn start(&self) {
        self.flow.graph.node_indices().for_each(|idx| self.start_component(idx));
    }

    pub fn stop(&self) {
        self.flow.graph.node_indices().for_each(|idx| self.stop_component(idx));
    }

    pub fn test(&self) {
        // Producer config

        // Only producers can be started / stopped

        // On start of producer calculate execution order (store in struct)
        // Retrieve items / optional batches in loop - producer sig change to Vec<FlowItem>
        // Execute each processor sending results to
    }

    // Start a component at a specified NodeIndex
    fn start_component(&self, node_idx: NodeIndex) {
        let component = self.flow.graph.node_weight(node_idx).unwrap();

        // Fetch the incoming / outgoing connections to / from the node
        let connections = self.flow.component_connections(node_idx);

        match component {
            Producer(producer) =>
                start_producer(Arc::clone(producer), connections),
            Processor(processor) =>
                start_processor(processor, connections)
        }
    }

    // Stop a component at a specified NodeIndex
    fn stop_component(&self, node_idx: NodeIndex) {
        let component = self.flow.graph.node_weight(node_idx).unwrap();

        match component {
            Producer(producer) => producer.control().pause(),
            Processor(processor) => processor.control().pause(),
        };
    }
}


fn start_processor(processor: &Arc<dyn Process>, connections: ComponentConnections) {
    for rx in connections.get_incoming_rx() {
        let processor_clone = processor.clone();
        let outgoing: Vec<Sender<FlowItem>> = connections.get_outgoing_tx();

        thread::spawn(move || loop {
            // End on component pause
            if processor_clone.control().is_paused() {
                return;
            }

            // Try and receive item from channel
            match rx.lock().unwrap().try_recv() {
                Ok(input) => {
                    let result = processor_clone.process(input).unwrap();

                    for tx in &outgoing {
                        tx.send(result.clone());
                    }
                }
                _ => {}
            }
        });
    }
}

fn start_producer(producer: Arc<dyn Produce>, connections: ComponentConnections) {
    let sleep = 1000 / producer.producer_config().count_per_second;

    thread::spawn(move || {
        loop {
            if producer.control().is_paused() {
                return;
            }

            let item = producer.try_produce().unwrap();

            for tx in &connections.get_outgoing_tx() {
                tx.send(item.clone());
            }

            thread::sleep(Duration::from_millis(sleep as u64));
        }
    });
}