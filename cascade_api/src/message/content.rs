

#[derive(Debug, Clone)]
pub enum Content {
    Memory { buffer: Vec<u8> },
    // File exists in a disk location
    Disk,
    Http { url: String },
}
