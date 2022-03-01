use clap::Parser;

use common::config::AppConfig;
use operations::run_operation;
use tracing::Level;

mod common;
mod operations;
mod protocol;

#[tokio::main]
async fn main() {
    let config = AppConfig::parse();
    initialize_global(&config);

    run_operation(&config).await;
}

fn initialize_global(config: &AppConfig) {
    initialize_logging(config.log_level());
}

fn initialize_logging(level: Level) {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_level(true)
        .with_max_level(level)
        .init();
}
