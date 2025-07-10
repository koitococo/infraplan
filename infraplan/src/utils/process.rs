use std::{
  ffi::OsStr,
  os::unix,
  path::{
    Path,
    // PathBuf
  },
  // str::FromStr,
};

use tokio::{io::AsyncWriteExt, process::Command};

// pub fn find_executable(name: &str, root: &str) -> anyhow::Result<Option<String>> {
//   let root = PathBuf::from_str(root)?;

//   for c in ["usr/local/bin", "bin", "sbin", "usr/bin", "usr/sbin"] {
//     let path = root.join(c).join(name);
//     if path.exists() {
//       return Ok(Some(path.to_string_lossy().to_string()));
//     };
//   }

//   Ok(None)
// }

// pub fn get_ld_lib_path(root: &str) -> anyhow::Result<String> {
//   let root = PathBuf::from_str(root)?;
//   let mut paths = Vec::new();

//   for c in ["usr/lib", "usr/lib64", "lib", "lib64", "usr/lib/x86_64-linux-gnu", "lib/x86_64-linux-gnu"] {
//     let path = root.join(c);
//     if path.exists() {
//       paths.push(path.to_string_lossy().to_string());
//     }
//   }

//   let joined = paths.join(":");
//   Ok(joined)
// }

// pub async fn run_command_with_root<Args: std::fmt::Debug + IntoIterator<Item = impl AsRef<OsStr>>>(
//   command: &str, args: Args, new_root: &str,
// ) -> anyhow::Result<(i32, String, String)> {
//   log::info!("Running command: {command} {args:?} in root: {new_root}");

//   let Some(exe) = find_executable(command, new_root)? else {
//     log::error!("Executable {command} not found in {new_root}");
//     anyhow::bail!("Executable {command} not found in {new_root}");
//   };
//   log::debug!("Found executable: {exe}");

//   let ld_path = get_ld_lib_path(new_root)?;
//   log::debug!("LD_LIBRARY_PATH: {ld_path}");

//   let envs = vec![("PATH", "/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"), ("LD_LIBRARY_PATH", &ld_path)];

//   run_command_with_env(exe, args, envs).await
// }

pub async fn run_command<
  Cmd: std::fmt::Display + AsRef<OsStr>,
  Args: std::fmt::Debug + IntoIterator<Item = impl AsRef<OsStr>>,
>(
  command: Cmd, args: Args,
) -> anyhow::Result<(i32, String, String)> {
  run_command_with::<_, _, Vec<(&str, &str)>, &str, &str>(command, Some(args), None, None, None).await
}

pub async fn run_command_with_env<
  Cmd: std::fmt::Display + AsRef<OsStr>,
  Args: std::fmt::Debug + IntoIterator<Item = impl AsRef<OsStr>>,
  Envs: std::fmt::Debug + IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
>(
  command: Cmd, args: Args, envs: Envs,
) -> anyhow::Result<(i32, String, String)> {
  run_command_with::<_, _, _, &str, &str>(command, Some(args), Some(envs), None, None).await
}

pub async fn run_command_with_input<I: AsRef<[u8]>>(
  command: &str, args: &[&str], input: I,
) -> anyhow::Result<(i32, String, String)> {
  run_command_with::<_, _, Vec<(&str, &str)>, _, &str>(command, Some(args), None, Some(input), None).await
}

pub async fn run_command_with_chroot(
  command: &str, args: &[&str], new_root: &str,
) -> anyhow::Result<(i32, String, String)> {
  run_command_with::<_, _, Vec<(&str, &str)>, &str, _>(command, Some(args), None, None, Some(new_root.to_owned())).await
}

pub async fn run_command_with<
  Cmd: std::fmt::Display + AsRef<OsStr>,
  Args: std::fmt::Debug + IntoIterator<Item = impl AsRef<OsStr>>,
  Envs: std::fmt::Debug + IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
  Input: AsRef<[u8]>,
  Chroot: AsRef<Path> + Send + Sync + 'static,
>(
  command: Cmd, args: Option<Args>, envs: Option<Envs>, input: Option<Input>, chroot: Option<Chroot>,
) -> anyhow::Result<(i32, String, String)> {
  log::info!("Running command: {command} {args:?}");

  let mut cmd = Command::new(&command);
  if let Some(args) = args {
    cmd.args(args);
  }
  if let Some(envs) = envs {
    cmd.envs(envs);
  }
  if let Some(new_root) = chroot {
    cmd.env("PATH", "/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    cmd.current_dir(&new_root);
    unsafe {
      cmd.pre_exec(move || {
        unix::fs::chroot(&new_root)?;
        std::env::set_current_dir("/")?;
        Ok(())
      });
    }
  }

  cmd.stdin(if input.is_some() {
    std::process::Stdio::piped()
  } else {
    std::process::Stdio::null()
  });
  cmd.stdout(std::process::Stdio::piped());
  cmd.stderr(std::process::Stdio::piped());

  let mut child = cmd.spawn()?;
  if let Some(input) = input {
    let Some(mut stdin) = Option::take(&mut child.stdin) else {
      anyhow::bail!("Failed to take child stdio");
    };

    stdin.write_all(input.as_ref()).await?;
    drop(stdin); // Close stdin to signal end of input
  }

  let Ok(output) = child.wait_with_output().await else {
    log::error!("Failed to run command: {command}");
    anyhow::bail!("Failed to run command: {command}");
  };

  let status = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();

  if status == 0 {
    log::info!("Command finished: {command}");
  } else {
    log::warn!("Command finished with non-zero status {status}: {command}\n{stderr}");
  }
  Ok((status, stdout, stderr))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_run_command() {
    let (code, stdout, stderr) = run_command("echo", &["Hello, World!"]).await.unwrap();
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert_eq!(stdout.trim(), "Hello, World!");
  }

  #[tokio::test]
  async fn test_run_command_with_input() {
    let (code, stdout, stderr) =
      run_command_with::<_, Vec<&str>, Vec<(String, String)>, _, &str>("cat", None, None, Some("Hello, World!"), None)
        .await
        .unwrap();
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert_eq!(stdout.trim(), "Hello, World!");
  }
}
