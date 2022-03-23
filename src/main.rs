#![feature(iter_intersperse)]
#![feature(integer_atomics)]

extern crate core;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::component::{Process, Produce};
use crate::flow::{Flow, run_flow};
use crate::generate_item::GenerateItem;
use crate::item::Item;
use crate::log_message::LogMessage;
use crate::update_properties::UpdateProperties;

mod component;
mod item;
mod flow;
mod generate_item;
mod update_properties;
mod log_message;
mod connection;

fn main() {
    let generate_item = GenerateItem { content: "con".to_string() };
    let update_properties = UpdateProperties {
        updates: HashMap::from([("item".to_string(), "value".to_string())])
    };

    let update_properties2 = UpdateProperties {
        updates: HashMap::from([("item2".to_string(), "otherValue".to_string())])
    };

    let log_message = LogMessage {};

    let time = SystemTime::now();

    let flow = Flow::new()
        .add_producer(Arc::new(generate_item))
        .connect_to_previous(Arc::new(update_properties))
        .connect_to_previous(Arc::new(update_properties2))
        .connect_to_previous(Arc::new(log_message));

    run_flow(&flow);

    thread::sleep(Duration::from_secs(100));

    let elapsed = time.elapsed().unwrap().as_millis();

    println!("Elapsed {}", elapsed)
}
