use std::ffi::CString;

use crate::utils::fstab::get_fstab_entries;

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

pub fn mount(blk: &str, target: &str, fstype: FsType) -> anyhow::Result<()> {
  let fstab = get_fstab_entries()?;
  if let Some(m) = fstab.iter().find(|v| v.mount_point == target) {
    log::info!("Target {} is already mounted, trying to unmount", target);
    unmount(m.mount_point.as_str())?;
  }

  let fstype: &str = fstype.into();
  log::info!("Mounting {} on {} as {}", blk, target, fstype);
  std::fs::create_dir_all(target)?;

  let r = unsafe {
    libc::mount(
      CString::new(blk)?.as_ptr(),
      CString::new(target)?.as_ptr(),
      CString::new(fstype)?.as_ptr(),
      libc::MS_MGC_MSK,
      std::ptr::null(),
    )
  };
  if r != 0 {
    let err = std::io::Error::last_os_error();
    log::error!("Failed to mount {} on {} as {}: {}", blk, target, fstype, &err);
    return Err(anyhow::anyhow!(err));
  }
  Ok(())
}

pub fn unmount(target: &str) -> anyhow::Result<()> {
  log::info!("Unmounting {}", target);
  let r = unsafe { libc::umount(CString::new(target)?.as_ptr()) };
  if r != 0 {
    let err = std::io::Error::last_os_error();
    log::error!("Failed to unmount {}: {}", target, &err);
    return Err(anyhow::anyhow!(err));
  }
  Ok(())
}

// pub fn chroot(target: &str) -> anyhow::Result<()> {
//   log::info!("Changing root to {}", target);
//   let r = unsafe { libc::chroot(CString::new(target)?.as_ptr()) };
//   if r != 0 {
//     let err = std::io::Error::last_os_error();
//     log::error!("Failed to change root to {}: {}", target, &err);
//     return Err(anyhow::anyhow!(err));
//   }
//   Ok(())
// }
