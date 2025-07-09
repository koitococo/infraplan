use crate::plugins::Distro;

pub mod tar;
mod utils;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommonConfig {
  pub disk: String,
  pub mount: String,
  pub distro: Distro,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Config {
  Tar(tar::Config),
}

pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    if *state {
      log::info!("Skipping sys_deploy plugin as it is already applied.");
      return Ok(());
    }
    match config {
      Config::Tar(inner) => tar::Context(self.0.clone()).invoke(inner, state).await?,
    }
    Ok(())
  }
}
