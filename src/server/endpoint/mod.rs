use std::collections::HashMap;
use hyper::{Body, header, http, Request, Response, StatusCode};
use hyper::body::{Buf, Bytes};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error;

pub(crate) mod create;
pub(crate) mod control;
pub(crate) mod list;

pub enum EndpointError {
    HyperError(hyper::Error),
    BadRequest(String),
    InternalServerError(String),
}

impl From<hyper::Error> for EndpointError {
    fn from(value: hyper::Error) -> Self {
        EndpointError::HyperError(value)
    }
}

impl From<http::Error> for EndpointError {
    fn from(value: http::Error) -> Self {
        EndpointError::BadRequest(value.to_string())
    }
}

impl From<Error> for EndpointError {
    fn from(value: Error) -> Self {
        EndpointError::BadRequest(value.to_string())
    }
}

pub type EndpointResult = Result<Response<Body>, EndpointError>;

pub async fn deserialise_body<T: DeserializeOwned>(
    request: Request<Body>,
) -> Result<T, EndpointError> {
    // This is actually Infallible
    let whole_body: Bytes = hyper::body::to_bytes(request).await.unwrap();

    Ok(serde_json::from_reader(whole_body.reader())?)
}

const APPLICATION_JSON: &str = "application/json";

// Serialise a value and return it in a JSON response body
fn create_json_body<T : Serialize>(to_serialise: &T) -> Result<Response<Body>, EndpointError> {
    match serde_json::to_string_pretty(to_serialise) {
        Ok(serialised) => {
            // Return serialised response in JSON body
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, APPLICATION_JSON)
                .body(Body::from(serialised))?)
        }
        Err(err) => Err(EndpointError::InternalServerError(err.to_string()))
    }
}

pub fn parse_query_params(request: Request<Body>) -> HashMap<String, String> {
    request
        .uri()
        .query()
        .map(|v| {
            // Use url lib to parse out query param
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        // Default to empty map instead of failure
        .unwrap_or_else(HashMap::new)
}
