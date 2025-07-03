#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  #[serde(rename = "type")]
  pub type_: String,
  pub interface: String,
  pub address: Option<String>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("Reconfigure Netplan with config: {:?}; globals: {:?}", self, global);
    Ok(())
  }
}
