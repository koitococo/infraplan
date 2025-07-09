use crate::utils::process::run_command;

const EXE_DNF: &str = "dnf";

pub async fn dnf_upgrade() -> anyhow::Result<()> {
  log::info!("Upgrading installed packages...");
  let (code, _, _) = run_command(EXE_DNF, &["upgrade", "-y"]).await?;

  if code != 0 {
    log::error!("Failed to upgrade packages with exit code: {code}");
    return Err(anyhow::anyhow!("Failed to upgrade packages"));
  }
  log::info!("Packages upgraded successfully.");
  Ok(())
}

pub async fn dnf_install(packages: &[String]) -> anyhow::Result<()> {
  if packages.is_empty() {
    log::warn!("No packages to install.");
    return Ok(());
  }

  log::info!("Installing packages: {}", packages.join(", "));
  let args: Vec<&str> = ["install", "-y"].into_iter().chain(packages.iter().map(|v| v.as_str())).collect();

  let (code, _, _) = run_command(EXE_DNF, &args).await?;

  if code != 0 {
    log::error!("Failed to install packages with exit code: {code}");
    return Err(anyhow::anyhow!("Failed to install packages"));
  }
  log::info!("Packages installed successfully.");
  Ok(())
}

pub async fn dnf_remove(packages: &[String]) -> anyhow::Result<()> {
  if packages.is_empty() {
    log::warn!("No packages to remove.");
    return Ok(());
  }

  log::info!("Removing packages: {}", packages.join(", "));
  let args: Vec<&str> = ["remove", "-y"].into_iter().chain(packages.iter().map(|v| v.as_str())).collect();

  let (code, _, _) = run_command(EXE_DNF, &args).await?;

  if code != 0 {
    log::error!("Failed to remove packages with exit code: {code}");
    return Err(anyhow::anyhow!("Failed to remove packages"));
  }
  log::info!("Packages removed successfully.");
  Ok(())
}
