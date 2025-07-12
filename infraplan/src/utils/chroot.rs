use crate::utils::{
  join_path_string,
  syscall::{FsType, mount, unmount},
};

pub fn prepare_chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Preparing chroot environment at {target}");
  let mounts = [
    ("tmp", FsType::Tmpfs),
    ("run", FsType::Tmpfs),
    ("proc", FsType::Proc),
    ("sys", FsType::Sysfs),
    ("dev", FsType::Devtmpfs),
    ("dev/pts", FsType::Devpts),
    ("dev/shm", FsType::Tmpfs),
    ("sys/firmware/efi", FsType::Efivarfs),
  ];
  for (path, fstype) in mounts {
    mount(None, join_path_string(target, path).as_str(), Some(fstype), false)?;
  }
  Ok(())
}

pub fn cleanup_chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Cleaning up chroot environment at {target}");
  let mounts = ["sys/firmware/efi", "dev/shm", "dev/pts", "dev", "sys", "proc", "run", "tmp"];
  for mount in mounts {
    let path = join_path_string(target, mount);
    if let Err(e) = unmount(&path) {
      log::warn!("Failed to unmount {path}: {e}");
    }
  }
  Ok(())
}
