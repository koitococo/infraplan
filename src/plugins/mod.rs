#![allow(async_fn_in_trait)]

pub mod package_manager;
pub mod reboot;
pub mod system_deployer;
pub mod system_reconfigurator;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub state_path: Option<String>,
  pub global: Option<Global>,
  pub recipe: Vec<Recipe>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Global {
  pub distro_hint: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
  pub id: String,
  pub name: Option<String>,
  pub chroot: Option<String>,
  pub overrides: Option<Global>,

  #[serde(flatten)]
  pub recipe_config: RecipeConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "use", content = "with")]
pub enum RecipeConfig {
  SystemDeployer(system_deployer::Config),
  PackageManager(package_manager::Config),
  Reboot(reboot::Config),
  SystemReconfigurator(system_reconfigurator::Config),
}

impl Config {
  pub fn to_json(&self) -> anyhow::Result<String> { serde_json::to_string(self).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_json(json: &str) -> anyhow::Result<Self> { serde_json::from_str(json).map_err(|e| anyhow::anyhow!(e)) }

  pub fn to_yaml(&self) -> anyhow::Result<String> { serde_yml::to_string(self).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_yaml(yaml: &str) -> anyhow::Result<Self> { serde_yml::from_str(yaml).map_err(|e| anyhow::anyhow!(e)) }

  pub fn from_path(path: &str) -> anyhow::Result<Self> {
    log::info!("Loading configuration from: {path}");
    let content = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!(e))?;
    if path.ends_with(".json") {
      Self::from_json(&content)
    } else if path.ends_with(".yaml") || path.ends_with(".yml") {
      Self::from_yaml(&content)
    } else {
      anyhow::bail!("Unsupported file format: {}", path);
    }
  }

  pub async fn invoke(&self) {
    log::debug!("Invoking configuration: {self:?}");
    for recipe in &self.recipe {
      if let Err(e) = recipe.invoke(self.global.as_ref().unwrap_or(&Global { distro_hint: None })).await {
        log::error!("Failed to invoke recipe {}: {}", recipe.name.as_deref().unwrap_or(&recipe.id), e);
        return;
      }
    }
  }
}

impl Global {
  pub fn clone_with_overrides(&self, overrides: &Option<Global>) -> Self {
    let Some(overrides) = overrides else {
      return self.clone();
    };
    let overrides = overrides.clone();
    let defaults = self.clone();
    Global {
      distro_hint: overrides.distro_hint.or(defaults.distro_hint),
    }
  }
}

impl Recipe {
  pub fn name(&self) -> &str { self.name.as_deref().unwrap_or(&self.id) }
}

pub trait Plugin {
  async fn invoke(&self, global: &Global) -> anyhow::Result<()>;
}

impl Plugin for RecipeConfig {
  async fn invoke(&self, global: &Global) -> anyhow::Result<()> {
    match self {
      RecipeConfig::SystemDeployer(config) => config.invoke(global).await,
      RecipeConfig::PackageManager(config) => config.invoke(global).await,
      RecipeConfig::Reboot(config) => config.invoke(global).await,
      RecipeConfig::SystemReconfigurator(config) => config.invoke(global).await,
    }
  }
}

impl Plugin for Recipe {
  async fn invoke(&self, global: &Global) -> anyhow::Result<()> {
    log::info!("Invoking recipe: {}", self.name());

    let global = global.clone_with_overrides(&self.overrides);
    if self.chroot.is_some() {
      anyhow::bail!("Chroot is not supported yet. Recipe: {}", self.name());
    }
    self.recipe_config.invoke(&global).await
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialize() {
    let config = Config {
      state_path: Some("/infraplan-state.json".to_string()),
      global: Some(Global {
        distro_hint: Some("ubuntu".to_string()),
      }),
      recipe: vec![
        Recipe {
          id: "system_deploy".to_string(),
          name: Some("Deploy ubuntu".to_string()),
          chroot: None,
          overrides: None,
          recipe_config: RecipeConfig::SystemDeployer(system_deployer::Config::Tar(system_deployer::tar::Config {
            url: "https://example.local/ubuntu.tar.gz".to_string(),
            common: system_deployer::CommonConfig {
              disk: "/dev/sda".to_string(),
              mount: "/mnt".to_string(),
            },
          })),
        },
        Recipe {
          id: "system_reconfig".to_string(),
          name: Some("Reconfigure system".to_string()),
          chroot: Some("/mnt".to_string()),
          overrides: None,
          recipe_config: RecipeConfig::SystemReconfigurator(vec![
            system_reconfigurator::ConfigItem::Netplan(vec![system_reconfigurator::netplan::ConfigItem {
              type_: "static".to_string(),
              interface: "eth0".to_string(),
              address: Some("172.16.1.1".to_string()),
            }]),
            system_reconfigurator::ConfigItem::User(vec![system_reconfigurator::user::ConfigItem {
              name: "ubuntu".to_string(),
              password: Some("password".to_string()),
              groups: Some(vec!["sudo".to_string(), "docker".to_string()]),
            }]),
            system_reconfigurator::ConfigItem::AptRepo(vec![system_reconfigurator::apt_repo::ConfigItem {
              overwrite: Some(true),
              name: Some("Ubuntu Archive".to_string()),
              base_url: "http://archive.ubuntu.com/ubuntu/".to_string(),
              distro: "focal".to_string(),
              components: vec!["main".to_string(), "universe".to_string()],
            }]),
          ]),
        },
        Recipe {
          id: "reboot".to_string(),
          name: Some("Reboot system".to_string()),
          overrides: None,
          chroot: None,
          recipe_config: RecipeConfig::Reboot(reboot::Config::Kexec(reboot::kexec::Config {
            linux: "/mnt/boot/vmlinuz".to_string(),
            initrd: "/mnt/boot/initrd.img".to_string(),
            root: "/dev/sda3".to_string(),
            append: Some("ro quiet splash".to_string()),
            move_state: Some("/mnt/infraplan-state.json".to_string()),
          })),
        },
        Recipe {
          id: "install_packages".to_string(),
          name: Some("Install packages".to_string()),
          overrides: None,
          chroot: None,
          recipe_config: RecipeConfig::PackageManager(package_manager::Config {
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
    println!("{}", json_content);

    let yaml_content = serde_yml::to_string(&config).expect("Failed to serialize to YAML");
    println!("{}", yaml_content);
  }

  #[test]
  fn deserialize_yaml() {
    let yaml_path = "examples/deploy_ubuntu.yaml";
    let yaml_content = std::fs::read_to_string(yaml_path).expect("Failed to read YAML file");
    let config: Config = serde_yml::from_str(&yaml_content).expect("Failed to deserialize YAML");
    println!("{:#?}", config);
  }
}
