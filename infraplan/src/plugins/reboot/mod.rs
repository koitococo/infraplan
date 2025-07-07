pub mod kexec;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Config {
  Kexec(kexec::Config),
}

impl super::Plugin for Config {
  type Context = super::Global;
  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    match self {
      Config::Kexec(inner) => inner.invoke(ctx).await,
    }
  }
}
