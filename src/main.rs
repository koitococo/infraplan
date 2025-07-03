use clap::Parser;

pub mod plugins;

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
}

#[derive(Parser, Debug)]
struct ApplyArgs {
  path: String,
}

#[tokio::main]
async fn main() {
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

  match cli.command {
    Command::Apply(args) => {
      log::info!("Applying configuration from path: {}", args.path);
      match plugins::Config::from_path(&args.path) {
        Ok(config) => config.invoke().await,
        Err(e) => log::error!("Error reading configuration: {e}"),
      }
    }
  }
}
