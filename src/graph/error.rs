use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum StartComponentError {
    InvalidNodeIndex(usize),
    MissingComponent(String),
}

impl Display for StartComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StartComponentError::InvalidNodeIndex(idx) => {
                f.write_fmt(format_args!("No node in graph at index {}", idx))
            }
            StartComponentError::MissingComponent(type_name) => f.write_fmt(format_args!(
                "Component {} not known to instance",
                type_name
            )),
        }
    }
}
