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
