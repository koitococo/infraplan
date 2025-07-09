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
  pub with: Vec<ConfigItem>,
}
pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = Vec<bool>;

  async fn invoke(&self, configs: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    let mut new_state = Vec::with_capacity(configs.with.len());
    for item in &configs.with {
      let mut state_i = state.pop().unwrap_or(false);
      match item {
        ConfigItem::Netplan(config) => netplan::Context(self.0.clone()).invoke(&config, &mut state_i).await?,
        ConfigItem::User(config) => {
          user::Context {
            globals: self.0.clone(),
            chroot: configs.chroot.clone(),
          }
          .invoke(&config, &mut state_i)
          .await?
        }
        ConfigItem::AptRepo(config) => {
          apt_repo::Context {
            globals: self.0.clone(),
            chroot: configs.chroot.clone(),
          }
          .invoke(&config, &mut state_i)
          .await?
        }
      }
      new_state.push(state_i);
    }
    *state = new_state;
    Ok(())
  }
}
