#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub overwrite: Option<bool>,
  pub name: Option<String>,
  pub base_url: String,
  pub distro: String,
  pub components: Vec<String>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  type Context = (Option<String>, crate::plugins::Global);
  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Reconfigure APT repos with config: {self:?}; globals: {ctx:?}");
    // TODO: implement APT repository reconfiguration logic here
    Ok(())
  }
}
