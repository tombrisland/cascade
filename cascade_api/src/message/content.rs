#[derive(Debug, Clone)]

pub enum Content {
    Memory { buffer: Vec<u8> },
    // File exists in an external disk location
    Disk,
    Http { url: String },
    // Within the content repository
    Local,
}
