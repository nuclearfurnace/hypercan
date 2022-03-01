use clap::{ArgEnum, Parser, Subcommand};
use tracing::Level;

#[derive(Parser)]
#[clap(name = "hypercan")]
#[clap(author = "Toby Lawrence <toby@nuclearfurnace.com>")]
#[clap(version = "0.1.0")]
#[clap(
    about = "A socketcan-based CLI for scanninng and querying OBD-II services/PIDs, UDS services, and more."
)]
pub struct AppConfig {
    /// Name of the socketcan device (i.e. `can0`) to open and use.
    socket_device_name: String,

    /// Logging level.  Defaults to INFO.
    #[clap(long, arg_enum, default_value_t = LogLevel::Info)]
    log_level: LogLevel,

    #[clap(subcommand)]
    operation: Command,
}

impl AppConfig {
    pub fn socket(&self) -> String {
        self.socket_device_name.clone()
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
    QueryAvailablePIDs {
        #[clap(long, short)]
        extended: bool,
    },
}
