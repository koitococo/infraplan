use tokio::process::Command;

pub async fn run_command(command: &str, args: &[&str]) -> anyhow::Result<(i32, String, String)> {
  log::info!("Running command: {} {:?}", command, args);
  let output = Command::new(command).args(args).output().await?;
  let status = output.status.code().unwrap_or(-1);
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  log::info!("Command finished with status {}: {}", status, stderr);
  Ok((status, stdout, stderr))
}
