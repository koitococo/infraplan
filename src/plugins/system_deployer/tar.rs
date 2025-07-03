#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub url: String,
  #[serde(flatten)]
  pub common: super::CommonConfig
}

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("System Deployer with config: {:?}; globals: {:?}", self, global);
    // TODO: implement system deployment logic here
    Ok(())
  }
}