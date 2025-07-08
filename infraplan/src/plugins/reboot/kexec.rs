use std::{path::PathBuf, str::FromStr};

use crate::utils::fstab::get_fstab_entries_by_path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub linux: String,
  pub initrd: String,
  pub root: String,
  pub append: Option<String>,
  pub move_state: Option<String>,
}

impl crate::plugins::Plugin for Config {
  type Context = crate::plugins::Global;

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Kexec with config: {self:?}; globals: {ctx:?}");
    // TODO: implement kexec logic here

    Ok(())
  }
}

pub fn find_kernel_parameters(new_root: &str) -> anyhow::Result<String> {
  let fstab = get_fstab_entries_by_path(PathBuf::from_str(new_root)?.join("etc/fstab"))?;
  let Some((root_device, root_options)) = fstab.iter().find_map(|v| {
    if v.mount_point != "/" {
      return None;
    }
    return Some((v.device.clone(), v.options.clone()));
  }) else {
    anyhow::bail!("No root filesystem found");
  };
  let root = if root_options.eq_ignore_ascii_case("defaults") {
    root_device
  } else {
    format!("{root_device} rootflags={root_options}")
  };

  let grub_config: Vec<(String, String)> =
    dotenvy::from_path_iter(PathBuf::from_str(new_root)?.join("etc/default/grub"))?
      .filter_map(|v| {
        let Ok(v) = v else {
          return None;
        };
        return Some(v);
      })
      .collect();

  let grub_cmdline = grub_config
    .iter()
    .find(|(k, _)| k == "GRUB_CMDLINE_LINUX")
    .map(|(_, v)| v.clone())
    .unwrap_or_else(|| String::from(""));

  let grub_cmdline_default = grub_config
    .iter()
    .find(|(k, _)| k == "GRUB_CMDLINE_LINUX_DEFAULT")
    .map(|(_, v)| v.clone())
    .unwrap_or_else(|| String::from(""));

  let params = format!("root={root} ro {grub_cmdline} {grub_cmdline_default}");
  Ok(params)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_find_kernel_parameters() {
    let root_path = "/";
    let params = find_kernel_parameters(root_path).unwrap();
    assert!(!params.is_empty(), "Kernel parameters should not be empty");
    println!("Kernel parameters: {params}");
  }
}
