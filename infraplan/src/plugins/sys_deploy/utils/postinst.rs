use std::process::exit;

use crate::{
  plugins::sys_deploy::Distro,
  utils::{
    chroot::{cleanup_chroot, invoke_chroot, prepare_chroot},
    process::run_command,
  },
};

pub async fn postinst(mountpoint: &str, distro: &Distro) -> anyhow::Result<()> {
  let child_pid = unsafe { libc::fork() };
  if child_pid == 0 {
    // Child process
    if prepare_chroot(mountpoint).is_err() {
      log::error!("Failed to prepare chroot environment at {}", mountpoint);
      exit(127);
    }
    if invoke_chroot(mountpoint).is_err() {
      log::error!("Failed to invoke chroot at {}", mountpoint);
      exit(126);
    }
    if after_chroot(distro).await.is_err() {
      log::error!("Failed to finalize chroot environment at {}", mountpoint);
      exit(1);
    }
    if cleanup_chroot(mountpoint).is_err() {
      log::error!("Failed to clean up chroot environment at {}", mountpoint);
      exit(125);
    }
    exit(0);
  } else {
    // Parent process
    Ok(())
  }
}

async fn after_chroot(distro: &Distro) -> anyhow::Result<()> {
  match distro {
    Distro::Ubuntu => postinst_ubuntu().await,
    _ => {
      // TODO: Implement post-installation steps for other distros
      log::warn!("No post-installation steps defined for distro: {:?}", distro);
      Ok(())
    }
  }
}

const EXE_UPDATE_INITRAMFS: &str = "update-initramfs";
const EXE_GRUB_INSTALL: &str = "grub-install";
const EXE_UPDATE_GRUB: &str = "update-grub";

async fn postinst_ubuntu() -> anyhow::Result<()> {
  run_command(EXE_UPDATE_INITRAMFS, &["-c", "-k", "all"]).await?;
  run_command(EXE_GRUB_INSTALL, &["--efi-directory=/boot/efi", "--recheck"]).await?;
  run_command(EXE_UPDATE_GRUB, &[]).await?;
  Ok(())
}
