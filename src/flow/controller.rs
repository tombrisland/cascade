use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use log::{debug, error, info, warn};
use petgraph::Direction;
use petgraph::graph::NodeIndex;
use tokio::time;

use crate::flow::connection::ComponentConnections;
use crate::flow::FlowComponent;
use crate::flow::item::FlowItem;
use crate::FlowGraph;
use crate::processor::Process;
use crate::producer::{Produce, Producer};

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

    async fn run_flow(&self, idx: NodeIndex, input: FlowItem) {
        let graph = &self.flow.graph;
        let neighbors = graph.neighbors_directed(idx, Direction::Outgoing);

        for neighbor_idx in neighbors {
            let component = graph.node_weight(neighbor_idx).unwrap();

            tokio::spawn(async move {
                if let FlowComponent::Processor(instance) = component {
                    let processor = instance.processor();

                    match processor.process(input.clone()) {
                        Ok(output) => {
                            self.run_flow(neighbor_idx, output);
                        }
                        Err(err) => {
                            error!("An error occurred");
                        }
                    }
                }
            });
        }
    }

    pub async fn start_flow(self) {
        let graph = self.flow.graph;

        // Get a list of producers
        let producers: Vec<NodeIndex> = Default::default();

        info!("Starting flow with {} entry points", producers.len());

        for idx in producers {
            let producer = graph.node_weight(idx).unwrap();

            let neighbors = graph.neighbors_directed(idx, Direction::Outgoing);

            for idx in neighbors {
                let processor = graph.node_weight(idx).unwrap();
            }
        }

        // Go through each possible processor
    }

    pub async fn start_producer(idx: NodeIndex, instance: Producer) {
        let producer = instance.producer();
        // Create an interval to produce at the configured rate
        let mut interval = time::interval(producer.config().schedule_duration());

        info!("Starting instance of producer {}", producer.name());

        let mut error_count: u8 = 0;

        // Spawn a new task to run the producer
        tokio::spawn(async move {
            loop {
                if instance.stop_requested() {
                    info!("Producer {} was terminated", instance.id);

                    return Result::Ok(());
                }

                match producer.try_produce() {
                    Ok(result) => {
                        debug!("Producer {} output item {}", instance.id, result.id);
                        // Callback to perform downstream processing
                        // TODO
                    }
                    Err(err) => {
                        // Disable the producer if it fails move than retry_count
                        if error_count >= producer.config().retry_count {
                            error!("Producer {} was terminated after failing {} times", instance.id, err);

                            return Result::Err(err);
                        }

                        warn!("Producer {} failed to run with {}", instance.id, err);
                        error_count += 1;
                    }
                };

                interval.tick().await;
            }
        });
    }

    // Start a component at a specified NodeIndex
    fn start_component(&self, node_idx: NodeIndex) {
        let component = self.flow.graph.node_weight(node_idx).unwrap();

        // Fetch the incoming / outgoing connections to / from the node
        let connections = self.flow.component_connections(node_idx);

        match component {
            FlowComponent::Producer(producer) =>
                start_producer(Arc::clone(producer), connections),
            FlowComponent::Processor(processor) =>
                start_processor(processor, connections)
        }
    }

    // Stop a component at a specified NodeIndex
    fn stop_component(&self, node_idx: NodeIndex) {
        let component = self.flow.graph.node_weight(node_idx).unwrap();

        match component {
            FlowComponent::Producer(producer) => producer.control().pause(),
            FlowComponent::Processor(processor) => processor.control().pause(),
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
    let sleep = 1000 / producer.config().count_per_second;

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