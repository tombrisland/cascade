use std::fs;
use std::collections::HashMap;
use std::fs::{DirEntry, Metadata};
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::component::{Control, Controllable};
use crate::flow::item::FlowItem;
use crate::producer::{Produce, ProduceError, ProducerConfig};

const NAME: &str = "GetFile";

pub struct GetFile {
    pub directory: Box<Path>,
    pub control: Control,
}

impl Controllable for GetFile {
    fn control(&self) -> &Control { &self.control }
}

impl Produce for GetFile {
    fn name(&self) -> &str {
        NAME
    }

    fn config(&self) -> ProducerConfig {
        ProducerConfig {
            count_per_second: 10,
            retry_count: 3
        }
    }

    fn try_produce(&self) -> Result<FlowItem, ProduceError> {
        let entries = fs::read_dir(&self.directory);

        // Error if the directory can't be read
        if entries.is_err() {
            return Result::Err(ProduceError {});
        }

        for entry in entries.unwrap() {
            let file = entry.unwrap();
            let metadata = file.metadata().unwrap();

            // Skip anything other than a file
            if !metadata.is_file() {
                // TODO support dir recursing
                continue;
            }

            return Result::Ok(FlowItem::new(file_properties(file, metadata)));
        }

        Result::Err(ProduceError {})
    }
}

fn file_properties(file: DirEntry, metadata: Metadata) -> HashMap<String, String> {
    HashMap::from([
        ("filename".to_string(), file.file_name().into_string().unwrap()),
        ("created".to_string(), metadata.created().unwrap()
            .duration_since(UNIX_EPOCH).unwrap().as_millis().to_string())
    ])
}