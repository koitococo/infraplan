use crate::utils::{
  join_path,
  process::run_command,
  syscall::{FsType, mount},
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
    let (code, _, stderr) = run_command(EXE_UDEVADM, &["trigger"]).await?;
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

pub async fn format_boot_part(part: &str) -> anyhow::Result<()> { format_ext4(part, "boot", Some(vec!["^metadata_csum_seed", "^orphan_file"])).await }

pub async fn format_root_part(part: &str) -> anyhow::Result<()> { format_ext4(part, "root", Some(vec!["^orphan_file"])).await }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PartedOutputs {
  disk: PartedDisk,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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
  filesystem: String,
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

pub async fn prepare_disk(disk: &str, use_mdev: bool, use_udev: bool, target: &str) -> anyhow::Result<()> {
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

  format_efi_part(efi_path.as_str()).await?;
  log::info!("Formatted EFI partition at {}", efi_path);
  format_boot_part(boot_path.as_str()).await?;
  log::info!("Formatted boot partition at {}", boot_path);
  format_root_part(rootfs_path.as_str()).await?;

  mount(rootfs_path.as_str(), target, FsType::Ext4)?;
  log::info!("Mounted root filesystem at {}", target);

  mount(boot_path.as_str(), join_path(target, "boot").as_ref(), FsType::Ext4)?;
  log::info!("Mounted boot partition at {}", join_path(target, "boot"));

  mount(efi_path.as_str(), join_path(target, "boot/efi").as_ref(), FsType::Vfat)?;
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
  std::fs::create_dir_all(std::path::Path::new(&fstab_path).parent().ok_or(anyhow::anyhow!("Failed to get parent directory"))?)?;
  std::fs::write(&fstab_path, fstab_content)?;
  log::info!("Wrote fstab to {}", &fstab_path);
  Ok(())
}
