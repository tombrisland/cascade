use crate::{Item, Process};
use crate::component::ProcessError;

pub struct LogMessage {
}

impl Process for LogMessage {
    fn process(&self, item: Item) -> Result<Item, ProcessError> {
        println!("LogMessage received {:?}", item);

        Result::Ok(item)
    }
}