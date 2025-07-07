#![allow(async_fn_in_trait)]

use std::path::Path;

pub mod pkgmgr;
pub mod reboot;
pub mod sys_deploy;
pub mod sysconf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub state_path: Option<String>,
  pub global: Option<Global>,
  pub recipe: Vec<Recipe>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Global {
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
pub struct Recipe {
  pub id: String,
  pub name: Option<String>,
  pub overrides: Option<Global>,

  #[serde(flatten)]
  pub recipe_config: RecipeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "use", content = "with")]
pub enum RecipeConfig {
  SystemDeployer(sys_deploy::Config),
  PackageManager(pkgmgr::Config),
  Reboot(reboot::Config),
  SystemReconfigurator(sysconf::Config),
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

  pub async fn invoke(&self) -> anyhow::Result<()> {
    log::debug!("Invoking configuration: {self:?}");
    for recipe in &self.recipe {
      if let Err(e) = recipe.invoke(self.global.as_ref().unwrap_or(&Global { distro_hint: None })).await {
        log::error!(
          "Failed to invoke recipe {}: {}",
          recipe.name.as_deref().unwrap_or(&recipe.id),
          e
        );
        return Err(e);
      }
    }
    Ok(())
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
  type Context;
  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()>;
}

impl Plugin for RecipeConfig {
  type Context = Global;

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    match self {
      RecipeConfig::SystemDeployer(config) => config.invoke(ctx).await,
      RecipeConfig::PackageManager(config) => config.invoke(ctx).await,
      RecipeConfig::Reboot(config) => config.invoke(ctx).await,
      RecipeConfig::SystemReconfigurator(config) => config.invoke(ctx).await,
    }
  }
}

impl Plugin for Recipe {
  type Context = Global;

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Invoking recipe: {}", self.name());

    let global = ctx.clone_with_overrides(&self.overrides);
    self.recipe_config.invoke(&global).await
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
      global: Some(Global {
        distro_hint: Some(Distro::Ubuntu),
      }),
      recipe: vec![
        Recipe {
          id: "system_deploy".to_string(),
          name: Some("Deploy ubuntu".to_string()),
          overrides: None,
          recipe_config: RecipeConfig::SystemDeployer(sys_deploy::Config::Tar(sys_deploy::tar::Config {
            url: "https://example.local/ubuntu.tar.zstd".to_string(),
            compression: Some(sys_deploy::tar::Compression::Zstd),
            common: sys_deploy::CommonConfig {
              disk: "/dev/sda".to_string(),
              mount: "/mnt".to_string(),
              distro: Distro::Ubuntu,
            },
          })),
        },
        Recipe {
          id: "system_reconfig".to_string(),
          name: Some("Reconfigure system".to_string()),
          overrides: None,
          recipe_config: RecipeConfig::SystemReconfigurator(sysconf::Config {
            chroot: Some("/mnt".to_string()),
            with: vec![
              sysconf::ConfigItem::Netplan(vec![sysconf::netplan::ConfigItem {
                type_: "static".to_string(),
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
        Recipe {
          id: "reboot".to_string(),
          name: Some("Reboot system".to_string()),
          overrides: None,
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
          recipe_config: RecipeConfig::PackageManager(pkgmgr::Config {
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
