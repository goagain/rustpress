use log::{Level, LevelFilter, Log, Metadata, Record};

static LOGGER: Logger = Logger;

pub struct Logger;

impl Logger {
    pub fn init() {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Info))
            .expect("Failed to initialize logger");
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}