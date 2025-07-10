#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  // #[serde(rename = "type")]
  // pub type_: String,
  pub dhcp: bool,
  pub interface: String,
  pub mac_address: String,
  pub address: Option<String>,
}

pub type Config = Vec<ConfigItem>;
pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, _config: &Self::Config, _state: &mut Self::State) -> anyhow::Result<()> {
    // TODO: implement logic here
    Ok(())
  }
}
