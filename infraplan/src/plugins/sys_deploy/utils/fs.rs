use crate::utils::{
  fstab::{find_mountpoint_by_device, is_mountpoint},
  join_path_string,
  parted_exe::{EXE_PARTED, get_parted_outputs},
  process::run_command,
  syscall::{FsType, mount, unmount_all},
};

const EXE_PARTPROBE: &str = "partprobe";
const EXE_MDEV: &str = "mdev";
const EXE_UDEVADM: &str = "udevadm";
const EXE_MKFS_VFAT: &str = "mkfs.vfat";
const EXE_MKFS_EXT4: &str = "mkfs.ext4";

pub async fn create_partition_table(disk: &str) -> anyhow::Result<()> {
  log::debug!("Creating partition table on disk {disk}");
  let args = vec![
    disk,       // block device to format
    "--script", // run in script mode
    "--fix",    // fix alignment issues
    "--align", "optimal", // set alignment
    "mklabel", "gpt", // create a new GPT partition table
    "mkpart", "primary", "fat32", "1MiB", "512MiB", // create a primary partition for EFI
    "mkpart", "primary", "ext4", "512MiB", "2048MiB", // create a primary partition for boot
    "mkpart", "primary", "ext4", "2048MiB", "100%", // create a primary partition for root
    "set", "1", "esp", "on", // set the first partition as ESP
  ];
  let (code, _, stderr) = run_command(EXE_PARTED, &args).await?;
  if code != 0 {
    anyhow::bail!("Failed to format disk {disk} with parted: {}", stderr);
  }
  Ok(())
}

pub async fn refresh_partition_table(disk: &str, use_mdev: bool, use_udev: bool) -> anyhow::Result<()> {
  log::debug!("Refreshing partition table for {disk}");
  let (code, _, stderr) = run_command(EXE_PARTPROBE, &[disk]).await?;
  if code != 0 {
    anyhow::bail!("Failed to refresh partitions for {disk}: {}", stderr);
  }
  if use_mdev {
    let (code, _, stderr) = run_command(EXE_MDEV, &["-s"]).await?;
    if code != 0 {
      anyhow::bail!("Failed to run mdev -s: {}", stderr);
    }
  }
  if use_udev {
    let (code, _, stderr) = run_command(EXE_UDEVADM, &["trigger", "--type=all", "--settle"]).await?;
    if code != 0 {
      anyhow::bail!("Failed to run udevadm trigger: {}", stderr);
    }
  }
  Ok(())
}

pub async fn format_efi_part(part: &str) -> anyhow::Result<()> {
  log::debug!("Formatting EFI partition {part}");
  let (code, _, stderr) = run_command(EXE_MKFS_VFAT, &["-F", "32", "-n", "EFI", part]).await?;
  if code != 0 {
    anyhow::bail!("Failed to format EFI partition {part}: {}", stderr);
  }
  Ok(())
}

pub async fn format_ext4(part: &str, label: &str, workarounds: Option<Vec<&str>>) -> anyhow::Result<()> {
  log::debug!("Formatting ext4 partition {part} with label {label}");
  let mut args = Vec::with_capacity(workarounds.as_ref().map_or(3, |w| w.len() * 2 + 3));
  args.push("-L");
  args.push(label);
  if let Some(workarounds) = workarounds {
    for workaround in workarounds {
      args.push("-O");
      args.push(workaround);
    }
  }
  args.push(part);
  let r = run_command(EXE_MKFS_EXT4, args).await?;
  if r.0 != 0 {
    anyhow::bail!("Failed to format ext4 partition {part} with label {label}: {}", r.2);
  }
  Ok(())
}

pub async fn format_boot_part(part: &str) -> anyhow::Result<()> {
  format_ext4(part, "boot", Some(vec!["^metadata_csum_seed", "^orphan_file"])).await
}

pub async fn format_root_part(part: &str) -> anyhow::Result<()> {
  format_ext4(part, "root", Some(vec!["^orphan_file"])).await
}

pub async fn block_for_disk_ready(disk: &str) -> anyhow::Result<()> {
  log::debug!("Blocking for disk {disk} to be ready");
  loop {
    let d = tokio::fs::try_exists(disk).await?;
    if d {
      break;
    }
    tokio::select! {
      _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {}
      _ = tokio::signal::ctrl_c() => {
        log::warn!("Interrupted while waiting for disk {disk} to be ready");
        return Err(anyhow::anyhow!("Interrupted while waiting for disk {disk} to be ready"));
      }
    }
  }
  Ok(())
}

pub async fn prepare_disk(disk: &str, use_mdev: bool, use_udev: bool, target: &str) -> anyhow::Result<()> {
  if is_mountpoint(target)? {
    unmount_all(target)?;
  }

  for mp in find_mountpoint_by_device(disk)? {
    log::info!("Found mount point for {disk}: {}", mp.mount_point);
    unmount_all(mp.mount_point.as_str())?;
  }

  create_partition_table(disk).await?;
  refresh_partition_table(disk, use_mdev, use_udev).await?;

  let parted = get_parted_outputs(disk).await?;
  let parts = parted.disk.partitions.iter().map(|v| v.uuid.clone()).collect::<Vec<_>>();
  if parts.len() != 3 {
    anyhow::bail!("Expected 3 partitions on {disk}, found {}", parts.len());
  }
  let efi_path = format!("/dev/disk/by-partuuid/{}", parts[0]);
  let boot_path = format!("/dev/disk/by-partuuid/{}", parts[1]);
  let rootfs_path = format!("/dev/disk/by-partuuid/{}", parts[2]);

  block_for_disk_ready(efi_path.as_str()).await?;
  format_efi_part(efi_path.as_str()).await?;
  log::info!("Formatted EFI partition at {efi_path}");

  block_for_disk_ready(boot_path.as_str()).await?;
  format_boot_part(boot_path.as_str()).await?;
  log::info!("Formatted boot partition at {boot_path}");

  block_for_disk_ready(rootfs_path.as_str()).await?;
  format_root_part(rootfs_path.as_str()).await?;
  log::info!("Formatted root filesystem at {rootfs_path}");

  mount(Some(rootfs_path.as_str()), target, Some(FsType::Ext4), false)?;
  log::info!("Mounted root filesystem at {target}");

  mount(
    Some(boot_path.as_str()),
    join_path_string(target, "boot").as_ref(),
    Some(FsType::Ext4),
    false,
  )?;
  log::info!("Mounted boot partition at {}", join_path_string(target, "boot"));

  mount(
    Some(efi_path.as_str()),
    join_path_string(target, "boot/efi").as_ref(),
    Some(FsType::Vfat),
    false,
  )?;
  log::info!("Mounted EFI partition at {}", join_path_string(target, "boot/efi"));
  Ok(())
}

pub async fn generate_fstab(disk: &str) -> anyhow::Result<String> {
  let parted = get_parted_outputs(disk).await?;
  let parts = parted.disk.partitions.iter().map(|v| v.uuid.clone()).collect::<Vec<_>>();
  if parts.len() != 3 {
    anyhow::bail!("Expected 3 partitions on {disk}, found {}", parts.len());
  }

  Ok(format!(
    r#" # Generated by InfraPlan
PARTUUID={} / ext4 defaults 0 1
PARTUUID={} /boot ext4 defaults 0 2
PARTUUID={} /boot/efi vfat defaults 0 2"#,
    parts[2], parts[1], parts[0]
  ))
}

pub async fn write_fstab(disk: &str, target: &str) -> anyhow::Result<()> {
  let fstab_content = generate_fstab(disk).await?;
  let fstab_path = join_path_string(target, "etc/fstab");
  std::fs::create_dir_all(
    std::path::Path::new(&fstab_path)
      .parent()
      .ok_or(anyhow::anyhow!("Failed to get parent directory"))?,
  )?;
  std::fs::write(&fstab_path, fstab_content)?;
  log::info!("Wrote fstab to {}", &fstab_path);
  Ok(())
}
