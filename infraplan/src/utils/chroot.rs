use std::os::unix;

use tokio::process::Command;

use crate::utils::{
  join_path,
  syscall::{FsType, mount, unmount},
};

pub fn prepare_chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Preparing chroot environment at {target}");
  mount(None, join_path(target, "tmp").as_str(), Some(FsType::Tmpfs), false)?;
  mount(None, join_path(target, "run").as_str(), Some(FsType::Tmpfs), false)?;
  mount(None, join_path(target, "proc").as_str(), Some(FsType::Proc), false)?;
  mount(None, join_path(target, "sys").as_str(), Some(FsType::Sysfs), false)?;
  mount(None, join_path(target, "dev").as_str(), Some(FsType::Devtmpfs), false)?;
  mount(None, join_path(target, "dev/pts").as_str(), Some(FsType::Devpts), false)?;
  mount(None, join_path(target, "dev/shm").as_str(), Some(FsType::Tmpfs), false)?;
  mount(
    None,
    join_path(target, "sys/firmware/efi").as_str(),
    Some(FsType::Efivarfs),
    false,
  )?;
  Ok(())
}

pub fn cleanup_chroot(target: &str) -> anyhow::Result<()> {
  log::info!("Cleaning up chroot environment at {target}");
  let mounts = ["sys/firmware/efi", "dev/shm", "dev/pts", "dev", "sys", "proc", "run", "tmp"];
  for mount in mounts {
    let path = join_path(target, mount);
    if let Err(e) = unmount(&path) {
      log::warn!("Failed to unmount {path}: {e}");
    }
  }
  Ok(())
}

pub async fn run_command_chroot(command: &str, args: &[&str], new_root: &str) -> anyhow::Result<(i32, String, String)> {
  log::info!("Running command: {command} {args:?}");
  let new_root = new_root.to_owned();
  let mut cmd = Command::new(command);
  cmd.args(args);
  cmd.env("PATH", "/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
  cmd.current_dir(new_root.clone());
  unsafe {
    cmd.pre_exec(move || {
      unix::fs::chroot(&new_root)?;
      std::env::set_current_dir("/")?;
      Ok(())
    });
  }
  let Ok(output) = cmd.output().await else {
    log::error!("Failed to run command: {command} {args:?}");
    anyhow::bail!("Failed to run command: {command} {args:?}");
  };
  let status = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  log::info!("Command finished with status {status}: {stderr}");
  Ok((status, stdout, stderr))
}
