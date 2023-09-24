use std::io::Error;

#[derive(Debug)]
pub enum ComponentError {
    ComponentShutdown,
    InputClosed,
    OutputClosed,
    MissingInput,
    MissingOutput(String),
    // Errors from underlying processor
    IOError(Error),
    RuntimeError(String),
}

impl From<Error> for ComponentError {
    fn from(value: Error) -> Self {
        ComponentError::IOError(value)
    }
}
