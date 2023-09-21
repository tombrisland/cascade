// Required to call trait fns dynamically
#![feature(fn_traits)]
#![feature(async_fn_in_trait)]
extern crate core;

use std::sync::Arc;

use log::LevelFilter;
use tokio::sync::Mutex;
use cascade_component::{NamedComponent, Process};

use cascade_component_std::generate_item::GenerateItem;
use cascade_component_std::log_message::LogMessage;
use cascade_component_std::update_properties::UpdateProperties;
use cascade_core::controller::CascadeController;
use cascade_core::registry::{ComponentMap, ComponentRegistry};
use cascade_http_server::CascadeServer;

use crate::logger::SimpleLogger;

mod logger;

static LOGGER: SimpleLogger = SimpleLogger;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), hyper::Error> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let mut components: ComponentMap = Default::default();

    components.insert(GenerateItem::type_name(), GenerateItem::create_from_json);
    components.insert(LogMessage::type_name(), LogMessage::create_from_json);
    components.insert(
        UpdateProperties::type_name(),
        UpdateProperties::create_from_json,
    );

    let controller: Arc<Mutex<CascadeController>> = Arc::new(Mutex::new(CascadeController::new(
        ComponentRegistry::new(components),
    )));

    let service = CascadeServer {
        addr: "127.0.0.1:3000".parse().unwrap(),
        controller,
    };

    service.start().await
}
