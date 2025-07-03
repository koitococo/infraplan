#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub overwrite: Option<bool>,
  pub name: Option<String>,
  pub base_url: String,
  pub distro: String,
  pub components: Vec<String>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("Reconfigure APT repos with config: {:?}; globals: {:?}", self, global);
    Ok(())
  }
}
