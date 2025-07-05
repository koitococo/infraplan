pub mod kexec;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Config {
  Kexec(kexec::Config),
}

impl super::Plugin for Config {
  async fn invoke(&self, global: &super::Global) -> anyhow::Result<()> {
    match self {
      Config::Kexec(inner) => inner.invoke(global).await,
    }
  }
}
