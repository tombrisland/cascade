use serde::Serialize;

pub mod definition;

#[derive(Clone, Serialize)]
pub struct ConnectionMetadata {
    pub id: String,
    pub name: String,
    pub capacity: usize,
}
