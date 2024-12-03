use anyhow::bail;
use anyhow::Context;
use directories_next::ProjectDirs;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use toml_edit::DocumentMut;

const DEFAULT_CONFIG: &str = include_str!("./default-config.toml");

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
    let project_dirs = ProjectDirs::from("", "", "imgchest-cli")
        .context("failed to determine application directory")?;
    let config_dir = project_dirs.config_dir();
    tokio::fs::create_dir_all(config_dir).await?;

    let config_path = config_dir.join("config.toml");
    let config_str = match tokio::fs::read_to_string(&config_path).await {
        Ok(config_str) => config_str,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut file = File::options()
                .write(true)
                .create_new(true)
                .open(&config_path)
                .await
                .context("failed to create default config")?;
            file.write_all(DEFAULT_CONFIG.as_bytes()).await?;
            file.flush().await?;
            file.sync_all().await?;

            String::new()
        }
        Err(error) => return Err(error).context("failed to write config"),
    };

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
                .context("failed to save config")?;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct UserConfig {
    document: DocumentMut,
}

impl UserConfig {
    fn new(input: &str) -> anyhow::Result<Self> {
        let document: DocumentMut = input.parse().context("failed to parse config")?;

        Ok(Self { document })
    }

    fn set_api_key(&mut self, api_key: String) -> anyhow::Result<()> {
        self.document
            .as_table_mut()
            .insert("api-key", api_key.into());
        Ok(())
    }

    fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        let temp_path = nd_util::with_push_extension(path, "tmp");
        std::fs::write(&temp_path, self.document.to_string().as_bytes())?;
        std::fs::rename(&temp_path, path)?;

        Ok(())
    }
}
