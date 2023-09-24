#[derive(Debug)]
pub enum ComponentError {
    InputClosed,
    OutputClosed,
    MissingInputConnection,
    ComponentShutdown,
    // No connection matching name
    MissingOutput(String),
    // Error from underlying processor
    Error(String),
}
