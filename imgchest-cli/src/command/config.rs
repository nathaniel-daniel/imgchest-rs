use anyhow::Context;
use directories_next::ProjectDirs;
use tokio::fs::File;

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
}

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "edit",
    description = "edit the config with the default text editor"
)]
pub struct EditOptions {}

pub async fn exec(options: Options) -> anyhow::Result<()> {
    let project_dirs = ProjectDirs::from("", "", "imgchest-cli")
        .context("failed to determine application directory")?;
    let data_dir = project_dirs.data_dir();
    tokio::fs::create_dir_all(data_dir).await?;

    let config_path = data_dir.join("config.toml");
    let _config_str = match tokio::fs::read_to_string(&config_path).await {
        Ok(config_str) => config_str,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            File::options()
                .write(true)
                .create_new(true)
                .open(&config_path)
                .await
                .context("failed to create empty config")?;

            String::new()
        }
        Err(error) => return Err(error).context("failed to write config"),
    };

    match options.subcommand {
        Subcommand::Edit(_options) => {
            opener::open(config_path)?;
        }
    }

    Ok(())
}
