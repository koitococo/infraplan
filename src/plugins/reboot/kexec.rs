#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub linux: String,
  pub initrd: String,
  pub root: String,
  pub append: Option<String>,
  pub move_state: Option<String>,
}

impl crate::plugins::Plugin for Config {
  async fn invoke(&self, global: &crate::plugins::Global) -> anyhow::Result<()> {
    log::info!("Kexec with config: {:?}; globals: {:?}", self, global);
    Ok(())
  }
}
