#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub install: Option<Vec<String>>,
  pub remove: Option<Vec<String>>,
  pub update: Option<bool>,
}

impl super::Plugin for Config {
  async fn invoke(&self, global: &super::Global) -> anyhow::Result<()> {
    log::info!("Package Manager with config: {self:?}; globals: {global:?}");
    // TODO: implement package management logic here
    Ok(())
  }
}
