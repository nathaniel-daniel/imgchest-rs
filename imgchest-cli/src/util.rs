use anyhow::Context;
use directories_next::ProjectDirs;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const DEFAULT_CONFIG: &str = include_str!("./default-config.toml");

/// Get the config dir
pub async fn get_config_dir() -> anyhow::Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "imgchest-cli")
        .context("failed to determine application directory")?;
    let config_dir = project_dirs.config_dir();
    tokio::fs::create_dir_all(config_dir).await?;

    Ok(config_dir.to_path_buf())
}

/// Get or init user config str
pub async fn read_or_init_user_config_str(config_path: &Path) -> anyhow::Result<String> {
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

    Ok(config_str)
}
