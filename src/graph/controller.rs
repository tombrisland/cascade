use std::fmt::{Display, Formatter};
use std::sync::Arc;

use log::{debug, info, warn};
use petgraph::graph::NodeIndex;
use tokio::time;
use tokio::time::MissedTickBehavior::Delay;

use crate::component::ComponentInstance;
use crate::component_registry::registry::ComponentRegistry;
use crate::connection::{Connection, Connections};
use crate::graph::graph::{CascadeGraph, ConnectionDetails};
use crate::graph::item::CascadeItem;
use crate::processor::{Process, Processor};
use crate::producer::{Produce, Producer};

// Control the execution of a FlowGraph
pub struct CascadeController {
    pub graph_definition: CascadeGraph,
}

#[derive(Debug)]
pub enum StartComponentError {
    InvalidNodeIndex(usize),
    MissingComponent(String),
}

impl Display for StartComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartComponentError::InvalidNodeIndex(idx) => f.write_fmt(format_args!("No node in graph at index {}", idx)),
            StartComponentError::MissingComponent(type_name) => f.write_fmt(format_args!("Component {} not known to instance", type_name)),
        }
    }
}

impl CascadeController {
    pub fn new(component_registry: ComponentRegistry) -> CascadeController {
        CascadeController {
            graph_definition: CascadeGraph { component_registry, graph_internal: Default::default(), connection_map: Default::default() },
        }
    }

    pub async fn start_component(&mut self, node_idx: NodeIndex) -> Result<(), StartComponentError> {
        let graph: &mut CascadeGraph = &mut self.graph_definition;

        // Fail if the index isn't present in the graph
        let (instance, metadata) = graph.get_component_for_idx(node_idx)
            .ok_or_else(|| StartComponentError::InvalidNodeIndex(node_idx.index()))?;

        info!("Starting {} component with id {}", metadata.type_name, metadata.id);

        let ConnectionDetails { incoming, outgoing } = graph.get_connections(node_idx);

        match instance {
            ComponentInstance::Producer(producer) => {
                tokio::spawn(async move {
                    // TODO no need to be arc
                    start_producer(Arc::new(producer), outgoing).await;
                });
            }
            ComponentInstance::Processor(processor) => {
                start_processor(Arc::new(processor), incoming, outgoing);
            }
        }

        Ok(())
    }
}

fn start_processor(instance: Arc<Processor>, incoming: Connections, outgoing: Connections) {
    // New reference to processor instance
    let instance: Arc<Processor> = Arc::clone(&instance);
    let outgoing: Arc<Connections> = Arc::new(outgoing);

    let incoming_connections = incoming.connections;

    // No point starting a task if this component has no input
    if incoming_connections.is_empty() {
        return;
    }

    for connection in incoming_connections {
        let instance: Arc<Processor> = Arc::clone(&instance);
        let incoming: Arc<Connection> = Arc::clone(&connection);
        let outgoing: Arc<Connections> = Arc::clone(&outgoing);

        // Task to execute processor on each input item
        tokio::spawn(async move {
            let processor: &Arc<dyn Process> = &instance.implementation;

            loop {
                if let Some(input) = incoming.recv().await {
                    let output: CascadeItem = processor.process(input).await.unwrap();

                    outgoing.send(output).await.expect("Something went wrong");
                }
            }
        });
    }
}

async fn start_producer(instance: Arc<Producer>, outbound: Connections) {
    // Create an interval to produce at the configured rate
    let mut interval = time::interval(instance.config.schedule_duration());

    // Don't try and catch up with missed ticks
    interval.set_missed_tick_behavior(Delay);

    loop {
        // New reference to producer instance
        let instance = Arc::clone(&instance);

        // Clone sender to be passed to tasks
        let outbound = outbound.clone();

        if instance.should_stop() {
            info!("Producer {} was terminated", instance.metadata.id);

            break;
        }

        interval.tick().await;

        // Skip adding another instance if there are too many
        if instance.concurrency_maxed() {
            continue;
        }

        instance.increment_concurrency();

        // Spawn a new task for each time the producer is scheduled
        tokio::spawn(async move {
            let producer = &instance.implementation;

            match producer.produce(outbound).await {
                // Producer returns count of items emitted to channel
                Ok(count) => {
                    debug!(
                            "Producer {} execution emitted {} items",
                            instance.metadata.id,
                            count
                        );
                }
                Err(err) => {
                    warn!("Producer failed to run with {}", err);
                }
            };

            instance.decrement_concurrency();
        });
    }
}