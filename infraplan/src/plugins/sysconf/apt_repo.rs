use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub overwrite: Option<bool>,
  pub name: Option<String>,
  pub base_url: String,
  pub distro: String,
  pub components: Vec<String>,
}

pub type Config = Vec<ConfigItem>;

pub struct Context {
  pub globals: crate::plugins::Globals,
  pub chroot: Option<String>,
}

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    if *state {
      log::info!("APT repositories are already configured");
      return Ok(());
    }

    log::info!("Configuring APT repositories...");
    let config_dir = PathBuf::from_str(self.chroot.as_ref().map(|v| v.as_str()).unwrap_or("/"))?.join("etc/apt");

    for item in config {
      let repo_file = match &item.name {
        Some(name) => format!("sources.list.d/{}.list", name),
        None => "sources.list".to_string(),
      };
      let file_path = config_dir.join(repo_file);
      if file_path.exists() {
        if !item.overwrite.unwrap_or(false) {
          log::info!("Skipping existing file {}", file_path.display());
          continue;
        }
      }

      let content = format!("deb {} {} {}\n", item.base_url, item.distro, item.components.join(" "));
      std::fs::write(&file_path, content)?;
    }

    *state = true;
    log::info!("APT repositories configured successfully");
    Ok(())
  }
}
