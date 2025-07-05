use std::ffi::CString;
// use infraplan_sys::{FstabEntry as FstabEntrySys, get_fstab_entries as get_fstab_entries_sys};

#[derive(Debug, Clone)]
pub struct FstabEntry {
  pub device: String,
  pub mount_point: String,
  pub file_system_type: String,
  pub options: String,
  pub dump: i32,
  pub pass: i32,
}



pub fn get_fstab_entries() -> Vec<FstabEntry> {
  let mut entries_ptr: *mut FstabEntrySys = std::ptr::null_mut();
  let result = unsafe { get_fstab_entries_sys(&mut entries_ptr) };
  if result == 0 {
    return Vec::with_capacity(0);
  }
  let mut entries = Vec::with_capacity(result as usize);
  unsafe {
    let mut current = entries_ptr;
    for _ in 0..result {
      let v = (*current).into();
      println!("Fstab Entry: {:?}", &v);
      entries.push(v);
      current = current.add(1);
    }
  }
  entries
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_fstab_entries() {
    let entries = get_fstab_entries();
    assert!(!entries.is_empty(), "Fstab entries should not be empty");
    for entry in entries {
      println!(
        "Device: {}, Mount Point: {}, FS Type: {}, Options: {}, Dump: {}, Pass: {}",
        entry.device, entry.mount_point, entry.file_system_type, entry.options, entry.dump, entry.pass
      );
    }
  }
}
