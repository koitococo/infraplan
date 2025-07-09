use crate::utils::process::run_command;

const EXE_APK: &str = "apk";

pub async fn apk_update() -> anyhow::Result<()> {
  log::info!("Updating package lists...");
  let (code, _, _) = run_command(EXE_APK, &["update"]).await?;

  if code != 0 {
    log::error!("Failed to update package lists with exit code: {}", code);
    return Err(anyhow::anyhow!("Failed to update package lists"));
  }
  log::info!("Package lists updated successfully.");
  Ok(())
}

pub async fn apk_upgrade() -> anyhow::Result<()> {
  log::info!("Upgrading installed packages...");
  let (code, _, _) = run_command(EXE_APK, &["upgrade", "--no-progress"]).await?;

  if code != 0 {
    log::error!("Failed to upgrade packages with exit code: {}", code);
    return Err(anyhow::anyhow!("Failed to upgrade packages"));
  }
  log::info!("Packages upgraded successfully.");
  Ok(())
}

pub async fn apk_install(packages: &[String]) -> anyhow::Result<()> {
  if packages.is_empty() {
    log::warn!("No packages to install.");
    return Ok(());
  }

  log::info!("Installing packages: {}", packages.join(", "));
  let args = ["add", "--no-progress"]
    .into_iter()
    .chain(packages.iter().map(|v| v.as_str()))
    .collect::<Vec<&str>>();

  let (code, _, _) = run_command(EXE_APK, &args).await?;

  if code != 0 {
    log::error!("Failed to install packages with exit code: {}", code);
    return Err(anyhow::anyhow!("Failed to install packages"));
  }
  log::info!("Packages installed successfully.");
  Ok(())
}

pub async fn apk_remove(packages: &[String]) -> anyhow::Result<()> {
  if packages.is_empty() {
    log::warn!("No packages to remove.");
    return Ok(());
  }

  log::info!("Removing packages: {}", packages.join(", "));
  let args = ["del", "--no-progress"]
    .into_iter()
    .chain(packages.iter().map(|v| v.as_str()))
    .collect::<Vec<&str>>();

  let (code, _, _) = run_command(EXE_APK, &args).await?;

  if code != 0 {
    log::error!("Failed to remove packages with exit code: {}", code);
    return Err(anyhow::anyhow!("Failed to remove packages"));
  }
  log::info!("Packages removed successfully.");
  Ok(())
}