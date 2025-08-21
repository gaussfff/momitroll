mod cli;
mod config;
mod printer;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use momitroll_config::Config;
use momitroll_core::migration::MigrationController;
use momitroll_logger::init_logger;
use printer::{print_info, print_version};

#[tokio::main]
async fn main() -> Result<()> {
    // init logger
    if let Err(e) = init_logger() {
        eprintln!("failed to initialize logger: {e}");
    }

    // process migration
    if let Err(e) = process_migration().await {
        eprintln!("failed to process migration: {e}");
    }

    Ok(())
}

async fn process_migration() -> Result<()> {
    // load config
    let config = Config::load()?;

    // migration controller
    let migration = MigrationController::new(config).await?;

    match Cli::parse().command {
        Command::Init => migration.init().await?,
        Command::Create { ref name } => migration.create(name).await?,
        Command::Up => migration.up().await?,
        Command::Down => migration.down().await?,
        Command::Status => migration.status().await?,
        Command::Drop => migration.drop().await?,
        Command::Info => print_info(),
        Command::Version => print_version(),
    }

    Ok(())
}
