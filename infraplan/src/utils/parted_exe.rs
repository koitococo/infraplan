use serde::{Deserialize, Serialize};

use crate::utils::process::run_command;

pub const EXE_PARTED: &str = "parted";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartedOutputs {
  pub disk: PartedDisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartedDisk {
  pub path: String,
  pub size: String,
  pub model: String,
  pub transport: String,
  pub logical_sector_size: i64,
  pub physical_sector_size: i64,
  pub label: String,
  pub uuid: String,
  pub max_partitions: i64,
  pub partitions: Vec<PartedPartition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PartedPartition {
  pub number: i64,
  pub start: String,
  pub end: String,
  pub size: String,
  #[serde(rename = "type")]
  pub partition_type: String,
  pub type_uuid: String,
  pub uuid: String,
  pub name: Option<String>,
  pub filesystem: Option<String>,
  pub flags: Option<Vec<String>>,
}

pub async fn get_parted_outputs(disk: &str) -> anyhow::Result<PartedOutputs> {
  log::debug!("Getting parted outputs for {disk}");
  let args = vec![disk, "--script", "print", "-j"];
  let (code, stdout, stderr) = run_command(EXE_PARTED, &args).await?;
  if code != 0 {
    anyhow::bail!("Failed to get parted outputs for {disk}: {}", stderr);
  }
  let outputs: PartedOutputs = serde_json::from_str(&stdout)?;
  Ok(outputs)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_deserialize_parted_outputs() {
    let output = r#"{"disk":{"path":"/dev/vdb","size":"10.7GB","model":"Virtio Block Device","transport":"virtblk","logical-sector-size":512,"physical-sector-size":512,"label":"gpt","uuid":"b1d47f57-77b8-4ce4-8f50-94f1a90e2ac4","max-partitions":128,"partitions":[{"number":1,"start":"1049kB","end":"537MB","size":"536MB","type":"primary","type-uuid":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","uuid":"7ad09dd4-bad0-4006-9af7-1981d0bc3c04","name":"primary","flags":["boot","esp"]},{"number":2,"start":"537MB","end":"2147MB","size":"1611MB","type":"primary","type-uuid":"0fc63daf-8483-4772-8e79-3d69d8477de4","uuid":"9ab37551-6643-4e99-bf30-ecfbcff3a08c","name":"primary"},{"number":3,"start":"2147MB","end":"10.7GB","size":"8589MB","type":"primary","type-uuid":"0fc63daf-8483-4772-8e79-3d69d8477de4","uuid":"dd539350-c688-4ed2-9bc8-621e89eb6bb0","name":"primary"}]}}"#;

    let parted_outputs: PartedOutputs = serde_json::from_str(output).expect("Failed to deserialize PartedOutputs");
    println!("{parted_outputs:#?}");
  }
}
