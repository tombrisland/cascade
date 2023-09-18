#[derive(Debug)]
pub enum ComponentError {
    InputClosed,
    OutputClosed,
    Missing,
    MissingInputConnection,
    // No connection matching name
    MissingOutputConnection(String),
    // Error from underlying processor
    Error(String),
}