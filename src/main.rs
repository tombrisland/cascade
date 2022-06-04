extern crate core;

use std::collections::HashMap;
use std::io::Read;
use std::iter::Map;
use std::net::SocketAddr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};

use ::log::LevelFilter;
use bytes::Buf;
use hyper::{Body, header, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::sync::oneshot::{channel, Receiver, Sender};
use crate::component::ComponentType;

use crate::flow::builder::FlowBuilder;
use crate::flow::controller::FlowController;
use crate::flow::FlowGraph;
use crate::logger::SimpleLogger;
use crate::processor::log_message::LogMessage;
use crate::processor::update_properties::UpdateProperties;
use crate::producer::generate_item::GenerateItem;

mod component;
mod flow;
mod logger;
mod processor;
mod producer;

static NOT_FOUND: &[u8] = b"Not Found";

static LOGGER: SimpleLogger = SimpleLogger;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

type Graphs = Arc<Mutex<HashMap<String, FlowGraph>>>;

#[derive(Serialize, Deserialize, Debug)]
struct CreateNode {
    component_name: String,
    component_type: ComponentType
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateGraph {
    graph_name: String
}

async fn create_graph(req: Request<Body>, graphs: Graphs) -> Result<Response<Body>> {
    let whole_body = hyper::body::aggregate(req).await?;

    let create_request: CreateGraph = serde_json::from_reader(whole_body.reader())?;

    FlowGraph::

    println!("{:?}", create_request);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("asd"))?;
    Ok(response)
}

async fn router(
    req: Request<Body>,
    graphs: Graphs
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/create_graph") => create_graph(req, graphs).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOT_FOUND.into())
                .unwrap())
        }
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Load all structs from list
    // Validate there are no duplicates

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let graphs : Graphs = Arc::new(Mutex::new(HashMap::new()));

    let service = make_service_fn(|_| {
        let graphs = Arc::clone(&graphs);
        async {
            Ok::<_, GenericError>(service_fn(move |req| {
                // Clone again to ensure that client outlives this closure.
                router(req, Arc::clone(&graphs))
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Logger failed to initialise");

    let generate_item = GenerateItem {
        batch_size: 10,
        content: Option::from("con".to_string()),
    };

    // let get_file = GetFile { directory: Box::from(Path::new("C:\\Users\\tombr\\Documents\\Code\\cascade\\src\\test")) };

    let update_properties = UpdateProperties {
        updates: HashMap::from([("item".to_string(), "value".to_string())]),
    };

    let log_message = LogMessage {
        item_count: AtomicUsize::new(1),
        log_every_x: 10_000,
    };

    let flow = FlowBuilder::new()
        .add_producer(generate_item)
        .connect_to_previous(update_properties)
        .connect_to_previous(log_message)
        .build();

    let mut controller = FlowController {
        flow,
        producer_handles: vec![],
    };

    controller.start_flow().await;

    let (tx, rx): (Sender<bool>, Receiver<bool>) = channel();

    rx.await;
}
