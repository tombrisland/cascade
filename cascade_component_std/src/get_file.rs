use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, Metadata, ReadDir};
use std::path::Path;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use cascade_api::component::{NamedComponent, Process};
use cascade_api::component::environment::ExecutionEnvironment;
use cascade_api::component::error::ComponentError;
use cascade_api::message::content::Content;
use cascade_api::message::Message;

#[derive(Deserialize)]
pub struct GetFile {
    // Amount of files to emit from each scheduled run
    pub batch_size: i32,
    // Path to poll for files
    pub path: Box<Path>,
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
        // Error if the directory can't be read
        let entries: ReadDir = fs::read_dir(&self.path)?;

        let mut files_read: i32 = 0;

        for entry in entries {
            // Break if we've read more files than batch_size
            if files_read >= self.batch_size {
                break;
            }

            let dir_entry: DirEntry = entry?;
            let metadata: Metadata = dir_entry.metadata()?;

            // Skip anything other than a file
            if !metadata.is_file() {
                // TODO support dir recursing
                continue;
            }

            let buffer: Vec<u8> = fs::read(dir_entry.path())?;

            let content: Content = Content::Memory { buffer };

            // Emit the file as a message
            execution
                .send_default(Message::new_with_content(
                    file_properties(dir_entry, metadata),
                    content,
                ))
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
