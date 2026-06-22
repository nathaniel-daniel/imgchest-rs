use crate::UserConfig;
use anyhow::Context;
use anyhow::bail;
use anyhow::ensure;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, clap::Parser)]
#[command(about = "Modify the cli config")]
pub struct Options {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Subcommand {
    Edit(EditOptions),
    Set(SetOptions),
}

#[derive(Debug, Clone, clap::Parser)]
#[command(about = "Edit the config with the default text editor")]
pub struct EditOptions {}

#[derive(Debug, Clone, clap::Parser)]
#[command(about = "Set a key value pair")]
pub struct SetOptions {
    #[arg(help = "The key to set")]
    pub key: String,

    #[arg(help = "The new value")]
    pub value: String,
}

pub async fn exec(options: Options) -> anyhow::Result<()> {
    let config_dir = crate::util::get_config_dir().await?;

    let config_path = config_dir.join("config.toml");
    let config_str = crate::util::read_or_init_user_config_str(&config_path).await?;

    match options.subcommand {
        Subcommand::Edit(_options) => {
            open(&config_path)?;
        }
        Subcommand::Set(options) => {
            let mut config = UserConfig::new(&config_str)?;

            match options.key.as_str() {
                "api-key" => {
                    config.set_api_key(options.value)?;
                }
                key => {
                    bail!("unknown key \"{key}\"");
                }
            }

            config
                .save_to_path(&config_path)
                .await
                .context("failed to save config")?;
        }
    }

    Ok(())
}

fn open(path: &Path) -> anyhow::Result<()> {
    if cfg!(target_os = "linux") {
        let status = Command::new("editor")
            .arg(path)
            .status()
            .context("failed to run \"editor\"")?;

        ensure!(status.success(), "bad exit code {status}");
    } else {
        opener::open(path)?;
    }

    Ok(())
}
