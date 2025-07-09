use tokio::{io::AsyncWriteExt, process::Command};

pub async fn run_command(command: &str, args: &[&str]) -> anyhow::Result<(i32, String, String)> {
  log::info!("Running command: {command} {args:?}");
  let Ok(output) = Command::new(command).args(args).output().await else {
    log::error!("Failed to run command: {command} {args:?}");
    anyhow::bail!("Failed to run command: {command} {args:?}");
  };
  let status = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  log::info!("Command finished with status {status}: {stderr}");
  Ok((status, stdout, stderr))
}

pub async fn run_command_with_input<I: AsRef<[u8]>>(
  command: &str, args: &[&str], input: I,
) -> anyhow::Result<(i32, String, String)> {
  log::info!("Running command: {command} {args:?}");

  let mut cmd = Command::new(command);
  cmd.args(args);
  cmd.stdin(std::process::Stdio::piped());
  cmd.stdout(std::process::Stdio::piped());
  cmd.stderr(std::process::Stdio::piped());

  let mut child = cmd.spawn()?;
  let Some(mut stdin) = child.stdin.take() else {
    anyhow::bail!("Failed to take child stdio");
  };

  stdin.write_all(input.as_ref()).await?;
  drop(stdin); // Close stdin to signal end of input

  let Ok(output) = child.wait_with_output().await else {
    log::error!("Failed to run command: {command} {args:?}");
    anyhow::bail!("Failed to run command: {command} {args:?}");
  };
  let status = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  log::info!("Command finished with status {status}: {stderr}");
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
    let (code, stdout, stderr) = run_command_with_input("cat", &[], "Hello, World!").await.unwrap();
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert_eq!(stdout.trim(), "Hello, World!");
  }
}
