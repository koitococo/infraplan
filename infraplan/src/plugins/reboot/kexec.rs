use std::{
  ffi::CString,
  fs,
  os::fd::AsRawFd,
  path::{Path, PathBuf},
  str::FromStr,
};

use crate::utils::fstab::get_fstab_entries_by_path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub linux: Option<String>,
  pub initrd: Option<String>,
  pub root: String,
  pub append: Option<String>,
  // pub move_state: Option<String>, // TODO: persistent states is not implemented yet
}

pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  /// Actually no returns when successful
  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    if *state {
      log::info!("Kexec already invoked");
      return Ok(());
    }
    *state = true;

    let (kernel, initramfs) = match (&config.linux, &config.initrd) {
      (Some(linux), Some(initrd)) => {
        let kernel = PathBuf::from_str(linux)?;
        if !kernel.exists() {
          anyhow::bail!("Specified kernel does not exist");
        }
        log::info!("Using specified kernel: {linux}");

        let initramfs = PathBuf::from_str(initrd)?;
        if !initramfs.exists() {
          anyhow::bail!("Specified initramfs does not exist");
        }
        log::info!("Using specified initramfs: {initrd}");

        (kernel, initramfs)
      }
      (Some(_), None) => {
        log::error!("Got specified kernel but no initramfs");
        anyhow::bail!("No initramfs specified");
      }
      (None, Some(_)) => {
        log::error!("Got specified initramfs but no kernel");
        anyhow::bail!("No kernel specified");
      }
      (None, None) => {
        log::warn!("No kernel or initramfs specified, trying to find in new root");
        let (kernel, initramfs) = find_kernel(&config.root)?.ok_or(anyhow::anyhow!("No kernel found"))?;
        log::info!("Using kernel: {}", kernel.display());
        log::info!("Using initramfs: {}", initramfs.display());
        (kernel, initramfs)
      }
    };
    let append = match &config.append {
      Some(args) => {
        log::info!("Using specified kernel parameters: {args}");
        args.to_string()
      }
      None => {
        log::info!("No kernel parameters specified, trying to find in new root");
        let params = find_kernel_params_grub(&config.root)?;
        log::info!("Kernel parameters: {params}");
        params
      }
    };
    let parmas = find_kernel_params_root(&config.root)?;
    let kernel_params = format!("{parmas} {append}");

    log::info!("Loading kernel and initramfs for kexec");
    kexec_file_load(&kernel, &initramfs, kernel_params)?;

    log::error!("Rebooting system using kexec");
    kexec_reboot()?;

    Ok(())
  }
}

/// See also: https://docs.kernel.org/admin-guide/kernel-parameters.html
pub fn find_kernel_params_root(new_root: &str) -> anyhow::Result<String> {
  let fstab = get_fstab_entries_by_path(PathBuf::from_str(new_root)?.join("etc/fstab"))?;
  let Some((root_device, root_options)) = fstab.iter().find_map(|v| {
    if v.mount_point != "/" {
      return None;
    }
    Some((v.device.clone(), v.options.clone()))
  }) else {
    anyhow::bail!("No root filesystem found");
  };
  let root = if root_options.eq_ignore_ascii_case("defaults") {
    root_device
  } else {
    format!("{root_device} rootflags={root_options}")
  };

  let params = format!("root={root} ro");
  Ok(params)
}

pub fn find_kernel_params_grub(new_root: &str) -> anyhow::Result<String> {
  let grub_config: Vec<(String, String)> =
    dotenvy::from_path_iter(PathBuf::from_str(new_root)?.join("etc/default/grub"))?
      .filter_map(|v| {
        let Ok(v) = v else {
          return None;
        };
        Some(v)
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

  Ok(format!("{grub_cmdline} {grub_cmdline_default}"))
}

fn is_file(item: &fs::DirEntry) -> anyhow::Result<bool> {
  let ftype = item.file_type()?;
  if ftype.is_file() {
    return Ok(true);
  }
  if ftype.is_symlink() {
    let c_path = fs::canonicalize(item.path())?;
    return Ok(c_path.is_file());
  }
  Ok(false)
}

fn list_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
  let files: Vec<PathBuf> = fs::read_dir(dir)?
    .filter_map(|v| {
      let Ok(v) = v else {
        return None;
      };
      if let Ok(true) = is_file(&v) {
        Some(v.path().to_path_buf())
      } else {
        None
      }
    })
    .collect();

  Ok(files)
}

fn sibling_file(path: &Path, name: &str) -> anyhow::Result<PathBuf> {
  let dir = path.parent().ok_or(anyhow::anyhow!("Path has no parent directory"))?;
  Ok(dir.join(name))
}

pub fn find_kernel(new_root: &str) -> anyhow::Result<Option<(PathBuf, PathBuf)>> {
  let boot_dir_path = PathBuf::from_str(new_root)?.join("boot");
  let mut files = list_files(&boot_dir_path)?;
  files.sort();
  files.reverse();
  if let Some(kernel) = files.iter().find(|v| v.file_name().is_some_and(|v| v == "vmlinuz" || v == "vmlinux")) {
    let initramfs = sibling_file(kernel, "initrd.img")?;
    if initramfs.exists() && initramfs.is_file() {
      return Ok(Some((kernel.clone(), initramfs.clone())));
    }

    let initramfs = sibling_file(kernel, "initramfs.img")?;
    if initramfs.exists() && initramfs.is_file() {
      return Ok(Some((kernel.clone(), initramfs.clone())));
    }
  }
  let pattern = regex::Regex::new(r#"(vmlinuz|vmlinux)-(.*)"#)?;
  for candidate in files.iter().filter(|v| {
    v.file_name()
      .is_some_and(|v| v.to_str().is_some_and(|s| s.starts_with("vmlinuz-") || s.starts_with("vmlinux-")))
  }) {
    let caps = pattern
      .captures(
        candidate
          .file_name()
          .ok_or(anyhow::anyhow!("Failed to get file name"))?
          .to_str()
          .ok_or(anyhow::anyhow!("Failed to convert file name to string"))?,
      )
      .ok_or(anyhow::anyhow!("Failed to capture version from file name"))?;
    let suffix = caps.get(2).ok_or(anyhow::anyhow!("Failed to get version suffix"))?.as_str();

    let initramfs = sibling_file(candidate, &format!("initrd-{suffix}.img"))?;
    if initramfs.exists() && initramfs.is_file() {
      return Ok(Some((candidate.clone(), initramfs)));
    }

    let initramfs = sibling_file(candidate, &format!("initramfs-{suffix}.img"))?;
    if initramfs.exists() && initramfs.is_file() {
      return Ok(Some((candidate.clone(), initramfs)));
    }
  }
  Ok(None)
}

/// See also: man kexec_file_load.2
pub fn kexec_file_load<P: AsRef<Path>>(kernel_path: P, initramfs_path: P, cmdline: String) -> anyhow::Result<()> {
  let kernel = fs::File::open(kernel_path.as_ref())?;
  let kernel_fd = kernel.as_raw_fd();

  let initramfs = fs::File::open(initramfs_path.as_ref())?;
  let initramfs_fd = initramfs.as_raw_fd();

  let c_cmdline = CString::new(cmdline)?;
  let cmdline_len = c_cmdline.as_bytes_with_nul().len() as u64;
  let c_cmdline_ptr = c_cmdline.as_ptr();

  let ret = unsafe {
    utils_sys::kexec_file_load(
      kernel_fd,
      initramfs_fd,
      cmdline_len,
      c_cmdline_ptr,
      0, // flags
    )
  };

  if ret != 0 {
    let err = std::io::Error::last_os_error();
    log::error!("kexec_reboot failed with error code {ret}: {err:?}");
    anyhow::bail!(err);
  }

  Ok(())
}

/// See also: man reboot.2
pub fn kexec_reboot() -> anyhow::Result<()> {
  let ret = unsafe { utils_sys::kexec_reboot() };
  if ret != 0 {
    let err = std::io::Error::last_os_error();
    log::error!("kexec_reboot failed with error code {ret}: {err:?}");
    anyhow::bail!(err);
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_find_kernel_parameters() {
    let root_path = "/";
    let params = find_kernel_params_root(root_path).unwrap();
    assert!(!params.is_empty(), "Kernel parameters should not be empty");
    println!("Kernel parameters: {params}");
  }

  #[test]
  fn test_find_kernel() {
    let new_root = "/";
    let result = find_kernel(new_root);
    assert!(result.is_ok());
    let (kernel, initramfs) = result.unwrap().unwrap();
    assert!(kernel.exists());
    assert!(initramfs.exists());
    println!("Found kernel: {kernel:?}, initramfs: {initramfs:?}");
  }
}
