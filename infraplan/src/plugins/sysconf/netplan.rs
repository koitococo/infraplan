#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  #[serde(rename = "type")]
  pub type_: String,
  pub interface: String,
  pub address: Option<String>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  type Context = (Option<String>, crate::plugins::Global);

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Reconfigure Netplan with config: {self:?}; globals: {ctx:?}");
    // TODO: implement Netplan reconfiguration logic here
    Ok(())
  }
}
