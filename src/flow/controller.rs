use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, error, info, warn};
use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time;
use crate::component::ComponentError;

use crate::flow::connection::FlowConnection;
use crate::flow::item::FlowItem;
use crate::flow::FlowComponent;
use crate::processor::Process;
use crate::producer::Producer;
use crate::FlowGraph;

// Control the execution of a FlowGraph
pub struct FlowController {
    pub flow: FlowGraph,

    pub producer_handles: Vec<JoinHandle<()>>,
}

impl FlowController {
    pub async fn start_flow(&mut self) {
        let graph = &self.flow.graph;

        // Get a list of producers in the flow
        let producers: &Vec<NodeIndex> = &self.flow.producer_indices;

        info!("Starting flow with {} entry point(s)", producers.len());

        for idx in producers {
            let idx = idx.clone();
            let component = graph.node_weight(idx).unwrap();

            if let FlowComponent::Producer(producer) = component {
                let (tx, mut rx): (Sender<FlowItem>, Receiver<FlowItem>) = channel(10_000);
                let graph = Arc::clone(graph);
                let producer = Arc::clone(producer);

                tokio::spawn(async move {
                    while let Some(item) = rx.recv().await {
                        let id = item.id;

                        match run_flow(&graph, idx, item).await {
                            Ok(_) => { debug!("Flow {} successfully completed for {}", "", id) }
                            Err(_) => { error!("Flow {} failed to complete for {} with {}", "", id, "") }
                        };
                    }
                });

                let handle = tokio::spawn(async move {
                    start_producer(producer, tx).await;
                });

                self.producer_handles.push(handle);
            }
        }
    }
}

async fn start_producer(instance: Arc<Producer>, tx: Sender<FlowItem>) {
    // Create an interval to produce at the configured rate
    let mut interval = time::interval(instance.config.schedule_duration());

    loop {
        // New reference to producer instance
        let instance = Arc::clone(&instance);

        // Clone sender to be passed to tasks
        let tx = tx.clone();

        if instance.should_stop() {
            info!("Producer {} was terminated", instance.implementation.id());

            break;
        }

        interval.tick().await;

        // Skip adding another instance if there are too many
        if instance.instances_maxed() {
            continue;
        }

        instance.add_instance();

        // Spawn a new task for each time the producer is scheduled
        tokio::spawn(async move {
            let producer = &instance.implementation;

            match producer.try_produce(tx).await {
                // Producer returns count of items emitted to channel
                Ok(result) => match result {
                    Some(count) => {
                        debug!(
                            "Producer {} execution emitted {} items",
                            producer.id(),
                            count
                        );
                    }
                    None => {
                        debug!("Producer {} execution emitted 0 items", producer.id());
                    }
                },
                Err(err) => {
                    warn!("Producer failed to run with {}", err);
                }
            };

            instance.remove_instance();
        });
    }
}

// Run the flow through recursively to the end
async fn run_flow(
    graph: &Arc<Graph<FlowComponent, FlowConnection>>,
    producer_index: NodeIndex,
    input: FlowItem,
) -> Result<(), ComponentError> {
    let mut queue: Vec<(Arc<FlowItem>, NodeIndex)> = vec![(Arc::new(input), producer_index)];

    // While there are executions left to schedule
    while !queue.is_empty() {
        if let Some((input, idx)) = queue.pop() {
            let component = graph.node_weight(idx).unwrap();

            let output: Result<Arc<FlowItem>, ComponentError> = match component {
                // For a producer pass through the input
                FlowComponent::Producer(_) => Result::Ok(input),

                // A processor can be scheduled to run and the output propagated
                FlowComponent::Processor(processor) => {
                    let processor = Arc::clone(&processor.implementation);

                    debug!("Processor {} will process an item", processor.id());

                    // Item borrowed from Arc and cloned before use
                    let borrowed: &FlowItem = input.borrow();
                    let input: FlowItem = borrowed.clone();

                    // Execute the processor's work in a task
                    let output =
                        tokio::spawn(async move { processor.try_process(input).await }).await.unwrap();

                    output.map(|output| Arc::new(output))
                }
            };

            // Handle any error from the processor that just executed
            match output {
                Ok(output) => {
                    // Count of processors immediately downstream
                    let mut count = 0;

                    graph
                        // Retrieve downstream connections
                        .neighbors_directed(idx, Direction::Outgoing)
                        // Add them to the executions list
                        .for_each(|idx| {
                            queue.push((output.clone(), idx));

                            count += 1;
                        });

                    debug!("Found {} components downstream", count);
                }
                Err(err) => {
                    error!("Processor {} failed to execute", err.component_id);

                    return Result::Err(err);
                }
            }
        }
    }

    Result::Ok(())
}
