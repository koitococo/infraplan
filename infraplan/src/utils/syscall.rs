use nix::mount::MsFlags;

use crate::utils::fstab::{get_fstab_entries, is_mountpoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
  Vfat,
  Ext4,
  Sysfs,
  Tmpfs,
  Proc,
  Devtmpfs,
  Devpts,
  Efivarfs,
}

impl From<FsType> for &'static str {
  fn from(value: FsType) -> Self {
    match value {
      FsType::Vfat => "vfat",
      FsType::Ext4 => "ext4",
      FsType::Sysfs => "sysfs",
      FsType::Tmpfs => "tmpfs",
      FsType::Proc => "proc",
      FsType::Devtmpfs => "devtmpfs",
      FsType::Devpts => "devpts",
      FsType::Efivarfs => "efivarfs",
    }
  }
}

pub fn mount(blk: Option<&str>, target: &str, fstype: Option<FsType>, flags: bool) -> anyhow::Result<()> {
  if is_mountpoint(target)? {
    log::info!("Target {target} is already mounted, trying to unmount");
    unmount(target)?;
  }
  std::fs::create_dir_all(target)?;

  if let Some(blk) = blk.as_ref() {
    log::info!("Mounting {blk} on {target} with fstype {:?}", fstype);
  } else {
    log::info!("Mounting {target} with fstype {:?}", fstype);
  }

  nix::mount::mount::<str, str, str, str>(
    blk,
    target,
    fstype.map(|fs| fs.into()),
    if flags { MsFlags::MS_MGC_MSK } else { MsFlags::empty() },
    None,
  )
  .map_err(|e| {
    log::error!("Failed to mount {blk:?} on {target}: {}", e);
    anyhow::anyhow!(e)
  })
}

pub fn unmount(target: &str) -> anyhow::Result<()> {
  log::info!("Unmounting {target}");
  nix::mount::umount(target).map_err(|e| {
    log::error!("Failed to unmount {}: {}", target, e);
    anyhow::anyhow!(e)
  })
}

pub fn unmount_all(target: &str) -> anyhow::Result<()> {
  log::info!("Unmounting all mounts on {target}");
  let mut fstab = get_fstab_entries()?;
  fstab.reverse();
  for entry in fstab {
    if entry.mount_point.starts_with(target) {
      unmount(entry.mount_point.as_str())?;
    }
  }
  Ok(())
}
