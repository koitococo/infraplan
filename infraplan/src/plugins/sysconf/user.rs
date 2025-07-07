#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub name: String,
  pub password: Option<String>,
  pub groups: Option<Vec<String>>,
}

pub type Config = Vec<ConfigItem>;

impl crate::plugins::Plugin for Config {
  type Context = (Option<String>, crate::plugins::Global);
  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Reconfigure users with config: {self:?}; globals: {ctx:?}");
    // TODO: implement user reconfiguration logic here
    Ok(())
  }
}
