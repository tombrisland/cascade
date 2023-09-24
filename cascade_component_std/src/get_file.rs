use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, Metadata};
use std::path::Path;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use cascade_api::component::{NamedComponent, Process};
use cascade_api::component::environment::ExecutionEnvironment;
use cascade_api::component::error::ComponentError;
use cascade_api::message::Message;

const ERR_CANNOT_READ_DIR: &str = "Unable to read directory";

#[derive(Deserialize)]
pub struct GetFile {
    // Amount of files to emit from each scheduled run
    pub batch_size: i32,
    // Directory to poll for files
    pub directory: Box<Path>,
}

impl NamedComponent for GetFile {
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        "GetFile"
    }
}

#[async_trait]
impl Process for GetFile {
    fn create_from_json(config: Value) -> Arc<dyn Process>
    where
        Self: Sized,
    {
        let get_file: GetFile = serde_json::from_value(config).unwrap();

        Arc::new(get_file)
    }

    async fn process(&self, execution: &mut ExecutionEnvironment) -> Result<(), ComponentError> {
        let entries = fs::read_dir(&self.directory);

        // Error if the directory can't be read
        if entries.is_err() {
            return Err(ComponentError::Error(ERR_CANNOT_READ_DIR.to_string()));
        }

        let mut files_read = 0;

        for entry in entries.unwrap() {
            // Break if we've read more files than batch_size
            if files_read >= self.batch_size {
                break;
            }

            let file = entry.unwrap();
            let metadata = file.metadata().unwrap();

            // TODO lock files by this process (can use self.id())?

            // Skip anything other than a file
            if !metadata.is_file() {
                // TODO support dir recursing
                continue;
            }

            // Emit the file as a message
            execution
                .send_default(Message::new(file_properties(file, metadata)))
                .await?;

            files_read += 1;
        }

        Ok(())
    }
}

fn file_properties(file: DirEntry, metadata: Metadata) -> HashMap<String, String> {
    HashMap::from([
        (
            "file_name".to_string(),
            file.file_name().into_string().unwrap(),
        ),
        (
            "file_created".to_string(),
            metadata
                .created()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        ),
    ])
}
