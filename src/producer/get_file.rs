use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, Metadata};
use std::path::Path;
use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::component::{Component, ComponentError};
use crate::flow::item::FlowItem;
use crate::producer::Produce;

const NAME: &str = "GetFile";

const ERR_CANNOT_READ_DIR: &str = "Unable to read directory";

pub struct GetFile {
    // Amount of files to emit from each scheduled run
    pub batch_size: i32,
    // Directory to poll for files
    pub directory: Box<Path>,
}

impl Component for GetFile {
    fn name(&self) -> &'static str {
        return NAME;
    }
}

#[async_trait]
impl Produce for GetFile {
    fn on_initialisation(&self) {}

    async fn try_produce(&self, tx: Sender<FlowItem>) -> Result<Option<i32>, ComponentError> {
        let entries = fs::read_dir(&self.directory);

        // Error if the directory can't be read
        if entries.is_err() {
            return Result::Err(ComponentError::new(self, ERR_CANNOT_READ_DIR.to_string()));
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

            // Emit the FlowItem on the channel
            match tx
                .send(FlowItem::new(file_properties(file, metadata)))
                .await
            {
                Ok(_) => {}
                Err(err) => {
                    return Result::Err(ComponentError::from_send_error(self, err));
                }
            };

            files_read += 1;
        }

        return Result::Ok(Option::Some(files_read));
    }
}

fn file_properties(file: DirEntry, metadata: Metadata) -> HashMap<String, String> {
    HashMap::from([
        (
            "filename".to_string(),
            file.file_name().into_string().unwrap(),
        ),
        (
            "created".to_string(),
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
