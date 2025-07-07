use clap::Parser;

use crate::{plugins::sys_deploy::tar::extract_tarball, utils::elevate_privileges};

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
  Apply(ApplyArgs),

  #[cfg(debug_assertions)]
  InternalTest(InternalTestArgs),
}

#[derive(Parser, Debug)]
struct ApplyArgs {
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
        if let Err(e) = config.invoke().await {
          log::error!("Error applying configuration: {e}");
        }
      }
      Err(e) => log::error!("Error reading configuration: {e}"),
    }
    Ok(())
  }
}

#[cfg(debug_assertions)]
impl InternalTestArgs {
  async fn run(&self) -> anyhow::Result<()> {
    log::info!("Running internal tests...");

    extract_tarball("/tmp/test.tar", "/tmp/dest", &None).await?;

    Ok(())
  }
}
