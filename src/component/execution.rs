use crate::connection::Connection;

pub struct ComponentExecution {
    incoming_connections: Vec<Connection>,
    outgoing_connections: Vec<Connection>,
}

impl ComponentExecution {
    // Get a single item from the session

    // Send an item onto the outbound queues
}