use crate::plugins::sys_deploy::utils::{prepare_disk, write_fstab};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub url: String,
  #[serde(flatten)]
  pub common: super::CommonConfig,
}

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("System Deployer with config: {self:?}; globals: {global:?}");
    // TODO: implement system deployment logic here

    let (use_mdev, use_udev) = match global.distro_hint.as_ref().map(|s| s.as_str()) {
      Some("alpine") => (true, false),                                                  // Alpine uses mdev
      Some("arch") | Some("debian") | Some("fedora") | Some("ubuntu") => (false, true), // Arch, Debian, Fedora, and Ubuntu use udev
      _ => {
        log::warn!("Unknown distro hint: {:?}, defaulting to no mdev or udev", global.distro_hint);
        (false, false)
      } // Unknown or unspecified distro, default to no mdev or udev
    };
    prepare_disk(self.common.disk.as_str(), use_mdev, use_udev, self.common.mount.as_str()).await?;
    write_fstab(self.common.disk.as_str(), self.common.mount.as_str()).await?;
    Ok(())
  }
}
