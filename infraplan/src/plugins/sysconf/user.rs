use crate::utils::process::{run_command, run_command_with_chroot, run_command_with_input};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConfigItem {
  pub name: String,
  pub password: Option<String>,
  pub groups: Option<Vec<String>>,
}

pub type Config = Vec<ConfigItem>;
pub struct Context {
  pub globals: crate::plugins::Globals,
  pub chroot: Option<String>,
}

const EXE_USERADD: &str = "useradd";
const EXE_CHPASSWD: &str = "chpasswd";

impl crate::plugins::Plugin for Context {
  type Config = Config;
  type State = bool;

  async fn invoke(&self, config: &Self::Config, state: &mut Self::State) -> anyhow::Result<()> {
    if *state {
      log::info!("User configuration is already applied.");
      return Ok(());
    }

    log::info!("Applying user configuration...");

    for item in config {
      if item.name == "root" {
        log::debug!("Skipping root user configuration");
        continue;
      }
      log::info!("Configuring user: {}", item.name);
      let mut useradd_args: Vec<&str> = vec![
        item.name.as_str(),
        "-m", // Create home directory
        "-s",
        "/bin/bash", // Default shell
      ];

      if let Some(groups) = &item.groups {
        for group in groups {
          useradd_args.push("-G"); // Add supplementary groups
          useradd_args.push(group.as_str());
        }
      }

      if let Some(new_root) = &self.chroot {
        // useradd_args.push("--root"); // Chroot before add user
        // useradd_args.push(new_root.as_str());
        // run_command_with_root(EXE_USERADD, &useradd_args, &new_root).await?;

        // In some distros (e.g. Fedora), the useradd command has compatibility issues with the distro in new root.
        // Simply tricks on linker may not works
        run_command_with_chroot(EXE_USERADD, &useradd_args, new_root).await?;
      } else {
        run_command(EXE_USERADD, &useradd_args).await?;
      }
    }

    let mut reset_passwd = Vec::new();
    for item in config {
      if let Some(password) = &item.password {
        reset_passwd.push(format!("{}:{}", item.name, password));
      }
    }
    if !reset_passwd.is_empty() {
      let args = if let Some(new_root) = &self.chroot {
        vec!["--root", new_root.as_str()]
      } else {
        vec![]
      };
      run_command_with_input(EXE_CHPASSWD, &args, reset_passwd.join("\n")).await?;
    }

    *state = true;
    log::info!("User configuration applied successfully.");
    Ok(())
  }
}
