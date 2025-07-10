// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct DiskInfo {
//   pub path: String,
//   pub size: u64,
//   pub model: String,
//   // pub transport: String,
//   pub logical_sector_size: u64,
//   pub physical_sector_size: u64,
//   pub label: Option<String>,
//   // pub uuid: String,
//   pub max_partitions: i64,
//   pub partitions: Vec<PartitionInfo>,
// }

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct PartitionInfo {
//   pub number: i32,
//   pub start: u64,
//   pub end: u64,
//   pub size: u64,
//   pub partition_type: String,
//   pub type_uuid: Option<String>,
//   pub uuid: Option<String>,
//   pub name: Option<String>,
//   pub filesystem: Option<String>,
//   pub flags: Vec<String>,
// }

// macro_rules! get_partition_flag {
//   ($flags:ident, $part:ident, $flag:expr, $name:expr) => {
//     if $part.is_flag_available($flag) && $part.get_flag($flag) {
//       $flags.push($name.to_string());
//     }
//   };
// }

// /// Gather disk information using libparted. Approximately equivalent to `parted --script unit s print -j` command output. Size is in sectors.
// pub fn gather_disk_info(path: &str) -> anyhow::Result<DiskInfo> {
//   let mut dev = libparted::Device::new(path)?;

//   let size = dev.length();
//   let logical_sector_size = dev.sector_size();
//   let physical_sector_size = dev.phys_sector_size();
//   let model = dev.model().to_string();

//   let disk = libparted::Disk::new(&mut dev)?;
//   let parts: Vec<libparted::Partition<'_>> = disk.parts().collect();

//   let partitions = parts
//     .into_iter()
//     .filter_map(|part| {
//       if part.num() < 0 {
//         return None;
//       }
//       let r = PartitionInfo {
//         number: part.num(),
//         start: part.geom_start() as u64,
//         end: part.geom_end() as u64,
//         size: part.geom_length() as u64,
//         partition_type: part.type_get_name().to_string(),
//         name: part.name().map(|s| s.to_string()),
//         filesystem: part.fs_type_name().map(|s| s.to_string()),
//         type_uuid: part.get_type_uuid().ok().map(|v| uuid::Uuid::from_bytes(v).to_string()),
//         uuid: part.get_uuid().ok().map(|v| uuid::Uuid::from_bytes(v).to_string()),
//         flags: {
//           let mut flags = Vec::with_capacity(0);
//           get_partition_flag!(flags, part, libparted::PartitionFlag::PED_PARTITION_BOOT, "boot");
//           get_partition_flag!(flags, part, libparted::PartitionFlag::PED_PARTITION_ESP, "esp");
//           flags
//         },
//       };
//       Some(r)
//     })
//     .collect();

//   Ok(DiskInfo {
//     path: path.to_string(),
//     logical_sector_size,
//     physical_sector_size,
//     model,
//     size,
//     partitions,
//     label: disk.get_disk_type_name().map(|v| v.to_string()),
//     max_partitions: {
//       let v = disk.max_partition_length();
//       if v == -1 { 128 } else { v }
//     },
//   })
// }
