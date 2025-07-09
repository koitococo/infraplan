use crate::plugins::Distro;

pub mod apk;
pub mod apt;
pub mod dnf;
pub mod pacman;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub install: Option<Vec<String>>,
  pub remove: Option<Vec<String>>,
  pub update: Option<bool>,
}

pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    if *state {
      log::info!("Package manager is already invoked, skipping.");
      return Ok(());
    }

    match self.0.distro_hint {
      Some(Distro::Debian) | Some(Distro::Ubuntu) => {
        log::info!("Using apt package manager for Debian/Ubuntu");

        apt::apt_update().await?;
        if config.update.unwrap_or(true) {
          apt::apt_upgrade().await?;
        }
        if let Some(install) = &config.install {
          apt::apt_install(install).await?;
        }
        if let Some(remove) = &config.remove {
          apt::apt_remove(remove).await?;
        }
      }
      Some(Distro::Fedora) => {
        log::info!("Using dnf package manager for Fedora");

        if config.update.unwrap_or(true) {
          dnf::dnf_upgrade().await?;
        }
        if let Some(install) = &config.install {
          dnf::dnf_install(install).await?;
        }
        if let Some(remove) = &config.remove {
          dnf::dnf_remove(remove).await?;
        }
      }
      Some(Distro::Arch) => {
        log::info!("Using pacman package manager for Arch Linux");

        pacman::pacman_update().await?;
        if config.update.unwrap_or(true) {
          pacman::pacman_update().await?;
        }
        if let Some(install) = &config.install {
          pacman::pacman_install(install).await?;
        }
        if let Some(remove) = &config.remove {
          pacman::pacman_remove(remove).await?;
        }
      }
      Some(Distro::Alpine) => {
        log::info!("Using apk package manager for Alpine Linux");

        apk::apk_update().await?;
        if config.update.unwrap_or(true) {
          apk::apk_update().await?;
        }
        if let Some(install) = &config.install {
          apk::apk_install(install).await?;
        }
        if let Some(remove) = &config.remove {
          apk::apk_remove(remove).await?;
        }
      }
      None => {
        anyhow::bail!("No distro hint provided for package manager plugin.");
      }
    }

    *state = true;
    Ok(())
  }
}
