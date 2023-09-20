#[derive(Debug)]
pub enum ComponentError {
    InputClosed,
    OutputClosed,
    MissingInputConnection,
    // No connection matching name
    MissingOutputConnection(String),
    // Error from underlying processor
    _Error(String),
}
