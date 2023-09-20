use log::{Level, Metadata, Record};

pub struct SimpleLogger;

const MISSING_MODULE: &str = "unknown_module";

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let module_path: &str = record.module_path().unwrap_or(MISSING_MODULE);

            println!("[{}] {} - {}", module_path, record.level(), record.args());
        }
    }

    fn flush(&self) {}
}