#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub url: String,
  #[serde(flatten)]
  pub common: super::CommonConfig
}
