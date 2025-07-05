use std::os::unix;

use crate::utils::{
  join_path,
  syscall::{FsType, mount, unmount},
};

pub fn prepare_chroot(target: &str) -> anyhow::Result<()> {
  // FIXME: Safety: The function assumes the target is not existed or a directory that is NOT a mountpoint. Checks should be added.
  log::info!("Preparing chroot environment at {}", target);
  mount("none", join_path(target, "tmp").as_str(), FsType::Tmpfs)?;
  mount("none", join_path(target, "run").as_str(), FsType::Tmpfs)?;
  mount("none", join_path(target, "proc").as_str(), FsType::Proc)?;
  mount("none", join_path(target, "sys").as_str(), FsType::Sysfs)?;
  mount("none", join_path(target, "dev").as_str(), FsType::Devtmpfs)?;
  mount("none", join_path(target, "dev/pts").as_str(), FsType::Devpts)?;
  mount("none", join_path(target, "dev/shm").as_str(), FsType::Tmpfs)?;
  mount("none", join_path(target, "sys/firmware/efi").as_str(), FsType::Efivarfs)?;
  Ok(())
}

pub fn chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Entering chroot environment at {}", target);
  unix::fs::chroot(target)?;
  std::env::set_current_dir("/")?;
  Ok(())
}

pub fn cleanup_chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Cleaning up chroot environment at {}", target);
  let mounts = ["tmp", "run", "proc", "sys", "dev", "dev/pts", "dev/shm", "sys/firmware/efi"];
  for mount in mounts {
    let path = join_path(target, mount);
    if let Err(e) = unmount(&path) {
      log::warn!("Failed to unmount {}: {}", path, e);
    }
  }
  Ok(())
}
