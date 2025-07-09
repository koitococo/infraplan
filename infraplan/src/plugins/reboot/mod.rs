pub mod kexec;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Config {
  Kexec(kexec::Config),
}

pub struct Context(pub crate::plugins::Globals);

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    match config {
      Config::Kexec(inner) => kexec::Context(self.0.clone()).invoke(inner, state).await,
    }
  }
}
