pub mod cli;
pub mod ghostty_adapter;
pub mod model;
pub mod recovery;
pub mod store;
pub mod tui;

use clap::Parser;

use crate::cli::{Cli, run_cli};
use crate::store::AppPaths;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("snapshot not found: {0}")]
    SnapshotNotFound(String),
    #[error("snapshot already exists: {0}")]
    SnapshotExists(String),
    #[error("invalid snapshot name: {0}")]
    InvalidSnapshotName(String),
    #[error("ghostty adapter error: {0}")]
    Adapter(String),
    #[error("tui error: {0}")]
    Tui(String),
}

pub fn run() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let paths = AppPaths::discover()?;
    run_cli(cli, paths)
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .try_init();
}
