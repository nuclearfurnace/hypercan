use std::time::Duration;

use clap::{ArgEnum, Args, Parser, Subcommand};
use tracing::Level;

use super::addressing::Addressing;

#[derive(Parser)]
#[clap(name = "hypercan")]
#[clap(author = "Toby Lawrence <toby@nuclearfurnace.com>")]
#[clap(version = "0.1.0")]
#[clap(
    about = "A socketcan-based CLI for scanninng and querying OBD-II services/PIDs, UDS services, and more."
)]
pub struct AppConfig {
    /// Logging level.  Defaults to INFO.
    #[clap(long, arg_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    #[clap(flatten)]
    can_parameters: CANParameters,

    #[clap(subcommand)]
    operation: Command,
}

impl AppConfig {
    pub fn can_parameters(&self) -> CANParameters {
        self.can_parameters.clone()
    }

    pub fn command(&self) -> Command {
        self.operation.clone()
    }

    pub fn log_level(&self) -> Level {
        match self.log_level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
        }
    }
}

#[derive(Args, Clone)]
pub struct CANParameters {
    #[clap(long)]
    pub socket_name: String,

    #[clap(long, parse(try_from_str = duration_str::parse), default_value = "2s")]
    pub read_timeout: Duration,

    #[clap(long, parse(try_from_str = duration_str::parse), default_value = "2s")]
    pub write_timeout: Duration,

    #[clap(long)]
    pub disable_isotp_frame_padding: bool,

    #[clap(long, default_value_t = 0xCC)]
    pub tx_frame_padding: u8,

    #[clap(long, short, arg_enum, default_value_t = Addressing::Standard)]
    pub addressing: Addressing,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Simply validates that the specific CAN socket can be opened.
    #[clap(name = "validate-socket")]
    ValidateSocket,

    /// Scans for available OBD-II PIDs.
    #[clap(name = "query-available-pids")]
    QueryAvailablePIDs,
}
