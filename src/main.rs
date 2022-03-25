extern crate core;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::thread;
use std::time::Duration;

use crate::component::Control;
use crate::flow::controller::FlowController;
use crate::flow::FlowGraph;
use crate::processor::log_message::LogMessage;
use crate::processor::update_properties::UpdateProperties;
use crate::producer::generate_item::GenerateItem;
use crate::producer::get_file::GetFile;

mod flow;
mod processor;
mod producer;
mod component;

fn main() {
    let generate_item = GenerateItem { content: "con".to_string(), control: Control::new() };

    let get_file = GetFile { directory: Box::from(Path::new("/Users/tomb/code/flow/src/test")), control: Control::new() };

    let update_properties = UpdateProperties {
        updates: HashMap::from([("item".to_string(), "value".to_string())]),
        control: Control::new(),
    };

    let log_message = LogMessage { counter: Arc::new(AtomicUsize::new(1)), control: Control::new() };

    let flow = FlowGraph::new()
        .add_producer(Arc::new(get_file))
        .connect_to_previous(Arc::new(update_properties))
        .connect_to_previous(Arc::new(log_message));

    let controller = FlowController { flow };

    controller.start();

    thread::sleep(Duration::from_secs(100));
}
