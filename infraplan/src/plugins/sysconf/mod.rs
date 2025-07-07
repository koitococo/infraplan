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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config { 
  pub chroot: Option<String>,
  pub with: Vec<ConfigItem>
}

impl super::Plugin for Config {
  type Context = super::Global;
  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    let ctx = (self.chroot.clone(), ctx.clone());
    for item in &self.with {
      match item {
        ConfigItem::Netplan(config) => config.invoke(&ctx).await?,
        ConfigItem::User(config) => config.invoke(&ctx).await?,
        ConfigItem::AptRepo(config) => config.invoke(&ctx).await?,
      }
    }
    Ok(())
  }
}
