use clap::Parser;

use crate::utils::elevate_privileges;

pub mod plugins;
pub mod utils;

#[derive(Parser, Debug)]
struct Cli {
  #[clap(subcommand)]
  command: Command,

  #[clap(long, short, default_value = "false")]
  verbose: bool,
}

#[derive(Parser, Debug)]
enum Command {
  /// Apply the configuration from the specified path.
  Apply(ApplyArgs),

  /// Recover states.
  Recover(RecoverArgs),

  #[cfg(debug_assertions)]
  InternalTest(InternalTestArgs),
}

#[derive(Parser, Debug)]
struct ApplyArgs {
  /// Path to the configuration file to apply.
  path: String,
}

#[derive(Parser, Debug)]
struct RecoverArgs {
  /// Path to the state file.
  path: String,
}

#[derive(Parser, Debug)]
struct InternalTestArgs {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or({
    #[cfg(debug_assertions)]
    {
      if cli.verbose { "trace" } else { "debug" }
    }
    #[cfg(not(debug_assertions))]
    {
      if cli.verbose { "debug" } else { "info" }
    }
  }))
  .format_timestamp_millis()
  .init();

  log::debug!("Parsed CLI arguments: {cli:?}");

  elevate_privileges()?;
  match cli.command {
    Command::Apply(args) => {
      args.run().await?;
    }
    Command::Recover(args) => {
      args.run().await?;
    }
    #[cfg(debug_assertions)]
    Command::InternalTest(args) => {
      args.run().await?;
    }
  }
  Ok(())
}

impl ApplyArgs {
  async fn run(&self) -> anyhow::Result<()> {
    log::info!("Applying configuration from path: {}", self.path);
    match plugins::Config::from_path(&self.path) {
      Ok(config) => {
        let mut state = config.into_state();
        state.invoke().await?;
        log::info!("Configuration applied successfully.");
      }
      Err(e) => log::error!("Error reading configuration: {e}"),
    }
    Ok(())
  }
}

impl RecoverArgs {
  async fn run(&self) -> anyhow::Result<()> {
    log::info!("Recovering states from path: {}", self.path);
    Ok(())
  }
}

#[cfg(debug_assertions)]
impl InternalTestArgs {
  async fn run(&self) -> anyhow::Result<()> {
    log::info!("Running internal tests...");

    Ok(())
  }
}
