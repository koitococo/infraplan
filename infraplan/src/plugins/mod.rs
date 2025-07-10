#![allow(async_fn_in_trait)]

use std::{collections::HashMap, path::Path};

pub mod pkgmgr;
pub mod reboot;
pub mod sys_deploy;
pub mod sysconf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub state_path: Option<String>, // TODO: implement persistent states
  pub global: Option<Globals>,
  pub recipe: Vec<RecipeConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Globals {
  pub distro_hint: Option<Distro>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distro {
  Ubuntu,
  Arch,
  Debian,
  Fedora,
  Alpine,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecipeConfig {
  pub id: String,
  pub name: Option<String>,
  pub overrides: Option<Globals>,

  #[serde(flatten)]
  pub config: PluginConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct State {
  pub config: Config,
  pub states: HashMap<String, RecipeState>,
  pub recipes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecipeState {
  pub id: String,
  pub display_name: String,
  pub global: Globals,
  pub config: PluginConfig,
  pub state: PluginState,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "use", content = "with")]
pub enum PluginConfig {
  SystemDeployer(<sys_deploy::Context as Plugin>::Config),
  PackageManager(<pkgmgr::Context as Plugin>::Config),
  Reboot(<reboot::Context as Plugin>::Config),
  SystemReconfigurator(<sysconf::Context as Plugin>::Config),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "state")]
pub enum PluginState {
  SystemDeployer(<sys_deploy::Context as Plugin>::State),
  PackageManager(<pkgmgr::Context as Plugin>::State),
  Reboot(<reboot::Context as Plugin>::State),
  SystemReconfigurator(<sysconf::Context as Plugin>::State),
}

impl Config {
  pub fn to_json(&self) -> anyhow::Result<String> { serde_json::to_string(self).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_json(json: &str) -> anyhow::Result<Self> { serde_json::from_str(json).map_err(|e| anyhow::anyhow!(e)) }

  pub fn to_yaml(&self) -> anyhow::Result<String> { serde_yml::to_string(self).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> { serde_yml::from_str(yaml).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
    log::info!("Loading configuration from: {}", path.as_ref().display());
    let content = std::fs::read_to_string(path.as_ref()).map_err(|e| anyhow::anyhow!(e))?;
    let ext_name = path.as_ref().extension().and_then(|s| s.to_str()).unwrap_or("");
    match ext_name {
      "json" => Self::from_json(&content),
      "yaml" | "yml" => Self::from_yaml(&content),
      _ => anyhow::bail!("Unsupported file format: {}", path.as_ref().display()),
    }
  }

  pub fn into_state(self) -> State {
    let mut states = HashMap::new();
    let mut recipes = Vec::new();
    for recipe in &self.recipe {
      recipes.push(recipe.id.clone());
      let state = recipe.into_state(&self.global);
      states.insert(state.id.clone(), state);
    }
    State {
      config: self,
      states,
      recipes,
    }
  }
}

impl RecipeConfig {
  pub fn into_state(&self, global: &Option<Globals>) -> RecipeState {
    let global = match (global, &self.overrides) {
      (Some(global), Some(overrides)) => Globals {
        distro_hint: overrides.distro_hint.as_ref().or(global.distro_hint.as_ref()).cloned(),
      },
      (Some(global), None) => global.clone(),
      (None, Some(overrides)) => overrides.clone(),
      (None, None) => Globals { distro_hint: None },
    };
    RecipeState {
      id: self.id.clone(),
      display_name: self.name.clone().unwrap_or_else(|| self.id.clone()),
      global,
      config: self.config.clone(),
      state: match &self.config {
        PluginConfig::SystemDeployer(_) => {
          PluginState::SystemDeployer(<sys_deploy::Context as Plugin>::State::default())
        }
        PluginConfig::PackageManager(_) => PluginState::PackageManager(<pkgmgr::Context as Plugin>::State::default()),
        PluginConfig::Reboot(_) => PluginState::Reboot(<reboot::Context as Plugin>::State::default()),
        PluginConfig::SystemReconfigurator(_) => {
          PluginState::SystemReconfigurator(<sysconf::Context as Plugin>::State::default())
        }
      },
    }
  }
}

pub trait Plugin {
  type Config;
  type State;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()>;
}

impl Plugin for Globals {
  type Config = PluginConfig;
  type State = PluginState;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    match config {
      PluginConfig::SystemDeployer(config) => {
        let state_i = match state {
          PluginState::SystemDeployer(s) => s,
          _ => {
            *state = PluginState::SystemDeployer(<sys_deploy::Context as Plugin>::State::default());
            match state {
              PluginState::SystemDeployer(s) => s,
              _ => unreachable!("State should have been set to SystemDeployer"),
            }
          }
        };
        sys_deploy::Context(self.clone()).invoke(config, state_i).await
      }
      PluginConfig::PackageManager(config) => {
        let state_i = match state {
          PluginState::PackageManager(s) => s,
          _ => {
            *state = PluginState::PackageManager(<pkgmgr::Context as Plugin>::State::default());
            match state {
              PluginState::PackageManager(s) => s,
              _ => unreachable!("State should have been set to PackageManager"),
            }
          }
        };
        pkgmgr::Context(self.clone()).invoke(config, state_i).await
      }
      PluginConfig::Reboot(config) => {
        let state_i = match state {
          PluginState::Reboot(s) => s,
          _ => {
            *state = PluginState::Reboot(<reboot::Context as Plugin>::State::default());
            match state {
              PluginState::Reboot(s) => s,
              _ => unreachable!("State should have been set to Reboot"),
            }
          }
        };
        reboot::Context(self.clone()).invoke(config, state_i).await
      }
      PluginConfig::SystemReconfigurator(config) => {
        let state_i = match state {
          PluginState::SystemReconfigurator(s) => s,
          _ => {
            *state = PluginState::SystemReconfigurator(<sysconf::Context as Plugin>::State::default());
            match state {
              PluginState::SystemReconfigurator(s) => s,
              _ => unreachable!("State should have been set to SystemReconfigurator"),
            }
          }
        };
        sysconf::Context(self.clone()).invoke(config, state_i).await
      }
    }
  }
}

impl RecipeState {
  pub async fn invoke(&mut self) -> anyhow::Result<()> {
    self.global.invoke(&self.config, &mut self.state).await.map_err(|e| anyhow::anyhow!(e))
  }
}

impl State {
  pub async fn invoke(&mut self) -> anyhow::Result<()> {
    for recipe_id in &self.recipes {
      log::info!("Invoking recipe: {recipe_id}");
      if let Some(recipe_state) = self.states.get_mut(recipe_id) {
        recipe_state.invoke().await?;
      } else {
        log::warn!("Recipe state for '{recipe_id}' not found");
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::{path::PathBuf, str::FromStr};

  use super::*;

  #[test]
  fn serialize() {
    let config = Config {
      state_path: Some("/infraplan-state.json".to_string()),
      global: Some(Globals {
        distro_hint: Some(Distro::Ubuntu),
      }),
      recipe: vec![
        RecipeConfig {
          id: "system_deploy".to_string(),
          name: Some("Deploy ubuntu".to_string()),
          overrides: None,
          config: PluginConfig::SystemDeployer(sys_deploy::Config::Tar(sys_deploy::tar::Config {
            url: "https://example.local/ubuntu.tar.zstd".to_string(),
            compression: Some(sys_deploy::tar::Compression::Zstd),
            common: sys_deploy::CommonConfig {
              disk: "/dev/sda".to_string(),
              mount: "/mnt".to_string(),
              distro: Distro::Ubuntu,
            },
          })),
        },
        RecipeConfig {
          id: "system_reconfig".to_string(),
          name: Some("Reconfigure system".to_string()),
          overrides: None,
          config: PluginConfig::SystemReconfigurator(sysconf::Config {
            chroot: Some("/mnt".to_string()),
            with: vec![
              sysconf::ConfigItem::Netplan(vec![sysconf::netplan::ConfigItem {
                dhcp: true,
                mac_address: "00:11:22:33:44:55".to_string(),
                interface: "eth0".to_string(),
                address: Some("172.16.1.1".to_string()),
              }]),
              sysconf::ConfigItem::User(vec![sysconf::user::ConfigItem {
                name: "ubuntu".to_string(),
                password: Some("password".to_string()),
                groups: Some(vec!["sudo".to_string(), "docker".to_string()]),
              }]),
              sysconf::ConfigItem::AptRepo(vec![sysconf::apt_repo::ConfigItem {
                overwrite: Some(true),
                name: Some("Ubuntu Archive".to_string()),
                base_url: "http://archive.ubuntu.com/ubuntu/".to_string(),
                distro: "focal".to_string(),
                components: vec!["main".to_string(), "universe".to_string()],
              }]),
            ],
          }),
        },
        RecipeConfig {
          id: "reboot".to_string(),
          name: Some("Reboot system".to_string()),
          overrides: None,
          config: PluginConfig::Reboot(reboot::Config::Kexec(reboot::kexec::Config {
            linux: Some("/mnt/boot/vmlinuz".to_string()),
            initrd: Some("/mnt/boot/initrd.img".to_string()),
            root: "/dev/sda3".to_string(),
            append: Some("ro quiet splash".to_string()),
            move_state: Some("/mnt/infraplan-state.json".to_string()),
          })),
        },
        RecipeConfig {
          id: "install_packages".to_string(),
          name: Some("Install packages".to_string()),
          overrides: None,
          config: PluginConfig::PackageManager(pkgmgr::Config {
            install: Some(vec![
              "vim".to_string(),
              "git".to_string(),
              "curl".to_string(),
              "wget".to_string(),
              "btop".to_string(),
              "docker.io".to_string(),
            ]),
            remove: Some(vec!["snapd".to_string(), "lxd".to_string()]),
            update: None,
          }),
        },
      ],
    };

    let json_content = serde_json::to_string_pretty(&config).expect("Failed to serialize to JSON");
    println!("{json_content}");
    let deserialized_json = Config::from_json(&json_content).expect("Failed to deserialize JSON");
    assert_eq!(config, deserialized_json);

    let yaml_content = serde_yml::to_string(&config).expect("Failed to serialize to YAML");
    println!("{yaml_content}");
    let deserialized_yaml = Config::from_yaml(&yaml_content).expect("Failed to deserialize YAML");
    assert_eq!(config, deserialized_yaml);

    let state = config.into_state();
    let state_json = serde_json::to_string_pretty(&state).expect("Failed to serialize state to JSON");
    println!("{state_json}");
    let deserialized_state: State = serde_json::from_str(&state_json).expect("Failed to deserialize state JSON");
    assert_eq!(state, deserialized_state);
  }

  #[test]
  fn deserialize_yaml() {
    const EXAMPLES_PATH: &str = "../examples";
    let examples_path = PathBuf::from_str(EXAMPLES_PATH).unwrap();
    let files: Vec<_> = examples_path.read_dir().unwrap().collect();

    for entry in files {
      let config_path = entry.unwrap().path();
      let config =
        Config::from_path(&config_path).unwrap_or_else(|_| panic!("Failed to load config: {}", config_path.display()));
      println!("{config:#?}");
    }
  }
}
