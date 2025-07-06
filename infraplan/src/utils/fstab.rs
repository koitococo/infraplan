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
