use std::net::SocketAddr;
use std::sync::Arc;

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use log::info;
use tokio::sync::{RwLock};

use cascade_core::controller::CascadeController;

use crate::endpoint::{EndpointError, EndpointResult};
use crate::endpoint::control::{start_component, stop_component};
use crate::endpoint::graph::{
    create_component, create_connection, list_graph_connections, list_graph_nodes,
    remove_component, remove_connection,
};
use crate::endpoint::metrics::stat_connection;
use crate::endpoint::registry::list_available_components;

mod endpoint;

static NOT_FOUND: &[u8] = b"Resource not found";

pub struct CascadeServer {
    pub addr: SocketAddr,

    pub controller: Arc<RwLock<CascadeController>>,
}

async fn router(
    controller: Arc<RwLock<CascadeController>>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let controller: Arc<RwLock<CascadeController>> = Arc::clone(&controller);

    let result: EndpointResult = match (req.method(), req.uri().path()) {
        // Retrieve information from the component registry
        (&Method::GET, "/list_available_components") => {
            list_available_components(controller, req).await
        }

        // Modify items in the graph
        (&Method::PUT, "/create_component") => create_component(controller, req).await,
        (&Method::PUT, "/create_connection") => create_connection(controller, req).await,
        (&Method::DELETE, "/remove_component") => remove_component(controller, req).await,
        (&Method::DELETE, "/remove_connection") => remove_connection(controller, req).await,

        // Control of individual components
        (&Method::GET, "/start_component") => start_component(controller, req).await,
        (&Method::GET, "/stop_component") => stop_component(controller, req).await,

        // List the current graph state
        (&Method::GET, "/list_nodes") => list_graph_nodes(controller, req).await,
        (&Method::GET, "/list_connections") => list_graph_connections(controller, req).await,
        (&Method::GET, "/stat_connection") => stat_connection(controller, req).await,
        // Return 404 not found response.
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(NOT_FOUND.into())
            .unwrap()),
    };

    // Handle any generic errors which fell through
    match result {
        Ok(response) => Ok(response),
        Err(err) => match err {
            EndpointError::HyperError(err) => Err(err),
            EndpointError::BadRequest(err) => Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(err.to_string()))
                .unwrap()),
            EndpointError::InternalServerError(err) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(err.to_string()))
                .unwrap()),
        },
    }
}

type ServerError = Box<dyn std::error::Error + Send + Sync>;

impl CascadeServer {
    pub async fn start(self) -> Result<(), hyper::Error> {
        info!("Starting server on {}", &self.addr);

        let service = make_service_fn(move |_| {
            let controller: Arc<RwLock<CascadeController>> = Arc::clone(&self.controller);

            async { Ok::<_, ServerError>(service_fn(move |req| router(controller.clone(), req))) }
        });

        let server = Server::bind(&self.addr).serve(service);

        info!("Server started on {}", &self.addr);

        server.await
    }
}
