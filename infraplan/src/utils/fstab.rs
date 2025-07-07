#[derive(Debug, Clone)]
pub struct FstabEntry {
  pub device: String,
  pub mount_point: String,
  pub file_system_type: String,
  pub options: String,
  pub dump: i32,
  pub pass: i32,
}

pub fn get_fstab_entries() -> anyhow::Result<Vec<FstabEntry>> {
  std::fs::read_to_string("/proc/self/mounts")
    .map(|contents| {
      contents
        .lines()
        .filter_map(|line| {
          let parts: Vec<&str> = line.split_whitespace().collect();
          if parts.len() == 6 {
            Some(FstabEntry {
              device: parts[0].into(),
              mount_point: parts[1].into(),
              file_system_type: parts[2].into(),
              options: parts[3].into(),
              dump: parts[4].parse().unwrap_or(-1),
              pass: parts[5].parse().unwrap_or(-1),
            })
          } else {
            None
          }
        })
        .collect()
    })
    .map_err(|e| anyhow::anyhow!("Failed to read /proc/self/mounts: {}", e))
}

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
  }
}
