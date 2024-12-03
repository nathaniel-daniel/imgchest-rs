use crate::UserConfig;
use anyhow::bail;
use anyhow::Context;

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "config", description = "modify the cli config")]
pub struct Options {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Edit(EditOptions),
    Set(SetOptions),
}

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "edit",
    description = "edit the config with the default text editor"
)]
pub struct EditOptions {}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand, name = "set", description = "set a key value pair")]
pub struct SetOptions {
    #[argh(positional, description = "the key to set")]
    pub key: String,

    #[argh(positional, description = "the new value")]
    pub value: String,
}

pub async fn exec(options: Options) -> anyhow::Result<()> {
    let config_dir = crate::util::get_config_dir().await?;

    let config_path = config_dir.join("config.toml");
    let config_str = crate::util::read_or_init_user_config_str(&config_path).await?;

    match options.subcommand {
        Subcommand::Edit(_options) => {
            opener::open(config_path)?;
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
