pub mod tar;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommonConfig {
  pub disk: String,
  pub mount: String,
} 

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Config {
  Tar(tar::Config),
}

impl super::Plugin for Config {
  async fn invoke(&self, global: &super::Global) -> anyhow::Result<()> {
    log::info!("System Deployer with config: {self:?}; globals: {global:?}");
    Ok(())
  }
}
