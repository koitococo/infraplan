#[derive(Debug, Clone)]
pub struct FstabEntry {
  pub device: String,
  pub mount_point: String,
  pub file_system_type: String,
  pub options: String,
  pub dump: i32,
  pub pass: i32,
}

pub fn get_fstab_entries_by_content(contents: String) -> Vec<FstabEntry> {
  contents
    .lines()
    .filter_map(|line| {
      let line = line.trim();
      if line.starts_with("#") {
        return None;
      }
      if line.is_empty() {
        return None;
      }
      let parts: Vec<&str> = line.trim().split_whitespace().collect();
      if parts.len() == 6 {
        let entry = FstabEntry {
          device: parts[0].into(),
          mount_point: parts[1].into(),
          file_system_type: parts[2].into(),
          options: parts[3].into(),
          dump: parts[4].parse().unwrap_or(-1),
          pass: parts[5].parse().unwrap_or(-1),
        };
        Some(entry)
      } else {
        None
      }
    })
    .collect()
}

pub fn get_fstab_entries_by_path<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Vec<FstabEntry>> {
  std::fs::read_to_string(path)
    .map(|contents| get_fstab_entries_by_content(contents))
    .map_err(|e| anyhow::anyhow!("Failed to read /proc/self/mounts: {}", e))
}

pub fn get_fstab_entries() -> anyhow::Result<Vec<FstabEntry>> { get_fstab_entries_by_path("/proc/self/mounts") }

fn canonicalized_path(path: &str) -> anyhow::Result<String> {
  let canonicalized =
    std::fs::canonicalize(path).map_err(|e| anyhow::anyhow!("Failed to canonicalize path {}: {}", path, e))?;
  Ok(canonicalized.to_string_lossy().to_string())
}

pub fn is_mountdevice(dev: &str) -> anyhow::Result<bool> {
  let fuz = dev.starts_with("/dev") && {
    let char = dev.as_bytes().last().unwrap_or(&b'\0');
    char <= &b'9' && char >= &b'0'
  };
  let c_dev = canonicalized_path(dev)?;
  Ok(get_fstab_entries()?.iter().any(|entry| {
    let dev = canonicalized_path(entry.device.as_str()).unwrap_or_default();
    (fuz && dev.starts_with(&c_dev)) || (!fuz && entry.device == c_dev)
  }))
}

pub fn is_mountpoint(path: &str) -> anyhow::Result<bool> {
  Ok(get_fstab_entries()?.iter().any(|entry| entry.mount_point == path))
}

pub fn get_entry_by_mountpoint(mountpoint: &str) -> anyhow::Result<Option<FstabEntry>> {
  let entries = get_fstab_entries()?;
  let c_mountpoint = canonicalized_path(mountpoint)?;
  Ok(entries.into_iter().find(|entry| entry.mount_point == c_mountpoint))
}

pub fn find_mountpoint_by_device(dev: &str) -> anyhow::Result<Vec<FstabEntry>> {
  let fuz = dev.starts_with("/dev") && {
    let char = dev.as_bytes().last().unwrap_or(&b'\0');
    char <= &b'9' && char >= &b'0'
  };
  let entries = get_fstab_entries()?;
  let c_dev = canonicalized_path(dev)?;

  let r: Vec<FstabEntry> = entries
    .into_iter()
    .filter(|entry| {
      let Ok(t_dev) = canonicalized_path(entry.device.as_str()) else {
        return false;
      };
      (fuz && t_dev.starts_with(&c_dev)) || (!fuz && entry.device == c_dev)
    })
    .collect();
  Ok(r)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_fstab_entries() {
    let entries = get_fstab_entries().unwrap();
    assert!(!entries.is_empty(), "Fstab entries should not be empty");
    for entry in entries {
      println!(
        "Device: {}, Mount Point: {}, FS Type: {}, Options: {}, Dump: {}, Pass: {}",
        entry.device, entry.mount_point, entry.file_system_type, entry.options, entry.dump, entry.pass
      );
    }

    let entries =get_fstab_entries_by_content(
      r#"
    # /etc/fstab
# Created by anaconda on Sun Jul 23 12:24:21 2023
#
# Accessible filesystems, by reference, are maintained under '/dev/disk/'.
# See man pages fstab(5), findfs(8), mount(8) and/or blkid(8) for more info.
#
# After editing this file, run 'systemctl daemon-reload' to update systemd
# units generated from this file.
#
UUID=84b9c9b5-f23a-4c2c-988d-2b8d51d15672   /           btrfs   subvol=root,compress=zstd:1   0 0 
UUID=ae667002-0448-4af0-8e4d-bc6c59c0a317   /boot       ext4    defaults                      1 2 
UUID=3C83-2ABC                              /boot/efi   vfat    umask=0077,shortname=winnt    0 2 
UUID=84b9c9b5-f23a-4c2c-988d-2b8d51d15672   /home       btrfs   subvol=home,compress=zstd:1   0 0
"#
      .to_string(),
    );
    assert!(!entries.is_empty(), "Fstab entries should not be empty");
    for entry in entries {
      println!(
        "Device: {}, Mount Point: {}, FS Type: {}, Options: {}, Dump: {}, Pass: {}",
        entry.device, entry.mount_point, entry.file_system_type, entry.options, entry.dump, entry.pass
      );
    }
  }
}
