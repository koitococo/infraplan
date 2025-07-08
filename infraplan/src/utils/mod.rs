use std::path::PathBuf;

use nix::unistd::Uid;

pub mod chroot;
pub mod fstab;
pub mod parted;
pub mod parted_exe;
pub mod process;
pub mod syscall;

pub fn join_path(base: &str, path: &str) -> String {
  let mut full_path: PathBuf = PathBuf::from(base);
  full_path.push(path);
  full_path.to_string_lossy().into_owned()
}

pub fn elevate_privileges() -> anyhow::Result<()> {
  let euid = nix::unistd::geteuid();
  if !euid.is_root() {
    nix::unistd::setuid(Uid::from_raw(0))?;
  }
  Ok(())
}