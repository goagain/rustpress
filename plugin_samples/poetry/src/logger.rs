use crate::rustpress::plugin::logger::{self, LogLevel as WitLevel};
use log::{Level, Metadata, Record};

static LOGGER: WasmLogger = WasmLogger {};

static INIT: std::sync::Once = std::sync::Once::new();
pub struct WasmLogger;

impl log::Log for WasmLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // here we can control the filtering, like default only show Info and above
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            // 1. convert the Rust log::Level to the WIT logger::Level
            let wit_level = match record.level() {
                Level::Error => WitLevel::Error,
                Level::Warn => WitLevel::Warn,
                Level::Info => WitLevel::Info,
                Level::Debug => WitLevel::Debug,
                Level::Trace => WitLevel::Trace,
            };

            // 2. format message (get the complete string from info!("Val: {}", x))
            let message = format!("{}", record.args());

            // 3. call Host function
            logger::log(wit_level, &message);
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    INIT.call_once(|| {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(log::LevelFilter::Trace))
            .expect("Logger initialization failed");

        log::info!("Logger initialized");
    });
}
