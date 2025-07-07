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

impl super::Plugin for Config {
  async fn invoke(&self, global: &super::Global) -> anyhow::Result<()> {
    match self {
      Config::Tar(inner) => inner.invoke(global).await,
    }
  }
}
