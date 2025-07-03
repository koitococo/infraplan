#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub name: String,
  pub password: Option<String>,
  pub groups: Option<Vec<String>>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("Reconfigure users with config: {self:?}; globals: {global:?}");
    Ok(())
  }
}
