use clap::Parser;

pub mod plugins;

#[derive(Parser, Debug)]
struct Cli {
  #[clap(subcommand)]
  command: Command,
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
  env_logger::init();
  log::set_max_level(log::LevelFilter::Trace);

  let cli = Cli::parse();
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
