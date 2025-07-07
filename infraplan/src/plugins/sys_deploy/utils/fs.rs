use crate::utils::{
  fstab::{find_mountpoint_by_device, is_mountpoint}, join_path, process::run_command, syscall::{mount, unmount_all, FsType}
};

const EXE_PARTED: &str = "parted";
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
  let r = run_command(EXE_MKFS_EXT4, args.as_ref()).await?;
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartedOutputs {
  disk: PartedDisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartedDisk {
  path: String,
  size: String,
  model: String,
  transport: String,
  logical_sector_size: i64,
  physical_sector_size: i64,
  label: String,
  uuid: String,
  max_partitions: i64,
  partitions: Vec<PartedPartition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartedPartition {
  number: i64,
  start: String,
  end: String,
  size: String,
  #[serde(rename = "type")]
  partition_type: String,
  type_uuid: String,
  uuid: String,
  name: Option<String>,
  filesystem: Option<String>,
  flags: Option<Vec<String>>,
}

pub async fn get_parted_outputs(disk: &str) -> anyhow::Result<PartedOutputs> {
  log::debug!("Getting parted outputs for {disk}");
  let args = vec![disk, "--script", "print", "-j"];
  let (code, stdout, stderr) = run_command(EXE_PARTED, &args).await?;
  if code != 0 {
    anyhow::bail!("Failed to get parted outputs for {disk}: {}", stderr);
  }
  let outputs: PartedOutputs = serde_json::from_str(&stdout)?;
  Ok(outputs)
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
    join_path(target, "boot").as_ref(),
    Some(FsType::Ext4),
    false,
  )?;
  log::info!("Mounted boot partition at {}", join_path(target, "boot"));

  mount(
    Some(efi_path.as_str()),
    join_path(target, "boot/efi").as_ref(),
    Some(FsType::Vfat),
    false,
  )?;
  log::info!("Mounted EFI partition at {}", join_path(target, "boot/efi"));
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
  let fstab_path = join_path(target, "etc/fstab");
  std::fs::create_dir_all(
    std::path::Path::new(&fstab_path)
      .parent()
      .ok_or(anyhow::anyhow!("Failed to get parent directory"))?,
  )?;
  std::fs::write(&fstab_path, fstab_content)?;
  log::info!("Wrote fstab to {}", &fstab_path);
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deserialize_parted_outputs() {
    let output = r#"{"disk":{"path":"/dev/vdb","size":"10.7GB","model":"Virtio Block Device","transport":"virtblk","logical-sector-size":512,"physical-sector-size":512,"label":"gpt","uuid":"b1d47f57-77b8-4ce4-8f50-94f1a90e2ac4","max-partitions":128,"partitions":[{"number":1,"start":"1049kB","end":"537MB","size":"536MB","type":"primary","type-uuid":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","uuid":"7ad09dd4-bad0-4006-9af7-1981d0bc3c04","name":"primary","flags":["boot","esp"]},{"number":2,"start":"537MB","end":"2147MB","size":"1611MB","type":"primary","type-uuid":"0fc63daf-8483-4772-8e79-3d69d8477de4","uuid":"9ab37551-6643-4e99-bf30-ecfbcff3a08c","name":"primary"},{"number":3,"start":"2147MB","end":"10.7GB","size":"8589MB","type":"primary","type-uuid":"0fc63daf-8483-4772-8e79-3d69d8477de4","uuid":"dd539350-c688-4ed2-9bc8-621e89eb6bb0","name":"primary"}]}}"#;

    let parted_outputs: PartedOutputs = serde_json::from_str(output).expect("Failed to deserialize PartedOutputs");
    println!("{parted_outputs:#?}");
  }
}
