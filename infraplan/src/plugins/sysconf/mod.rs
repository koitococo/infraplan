pub mod apt_repo;
pub mod netplan;
pub mod user;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "use", content = "with")]
pub enum ConfigItem {
  Netplan(netplan::Config),
  User(user::Config),
  AptRepo(apt_repo::Config),
}

pub type Config = Vec<ConfigItem>;

impl super::Plugin for Config {
  async fn invoke(&self, global: &super::Global) -> anyhow::Result<()> {
    for item in self {
      match item {
        ConfigItem::Netplan(config) => config.invoke(global).await?,
        ConfigItem::User(config) => config.invoke(global).await?,
        ConfigItem::AptRepo(config) => config.invoke(global).await?,
      }
    }
    Ok(())
  }
}
