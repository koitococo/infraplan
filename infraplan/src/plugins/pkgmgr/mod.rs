#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub install: Option<Vec<String>>,
  pub remove: Option<Vec<String>>,
  pub update: Option<bool>,
}

impl super::Plugin for Config {
  type Context = super::Global;

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("Package Manager with config: {self:?}; globals: {ctx:?}");
    // TODO: implement package management logic here
    Ok(())
  }
}
