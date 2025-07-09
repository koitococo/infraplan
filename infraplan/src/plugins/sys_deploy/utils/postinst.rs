use crate::{
  plugins::sys_deploy::Distro,
  utils::chroot::{cleanup_chroot, prepare_chroot, run_command_chroot},
};

pub async fn postinst(mountpoint: &str, distro: &Option<Distro>) -> anyhow::Result<()> {
  prepare_chroot(mountpoint)?;
  match distro {
    Some(Distro::Ubuntu) => postinst_ubuntu(mountpoint).await?,
    _ => {
      // TODO: Implement post-installation steps for other distros
      log::warn!("No post-installation steps defined for distro: {distro:?}");
    }
  }
  cleanup_chroot(mountpoint)?;
  Ok(())
}

const EXE_UPDATE_INITRAMFS: &str = "update-initramfs";
const EXE_GRUB_INSTALL: &str = "grub-install";
const EXE_UPDATE_GRUB: &str = "update-grub";

async fn postinst_ubuntu(new_root: &str) -> anyhow::Result<()> {
  run_command_chroot(EXE_UPDATE_INITRAMFS, &["-c", "-k", "all"], new_root).await?;
  run_command_chroot(EXE_GRUB_INSTALL, &["--efi-directory=/boot/efi", "--recheck"], new_root).await?;
  run_command_chroot(EXE_UPDATE_GRUB, &[], new_root).await?;
  Ok(())
}
