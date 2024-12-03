use anyhow::Context;
use std::path::Path;
use toml_edit::DocumentMut;

/// The user's config file
#[derive(Debug)]
pub struct UserConfig {
    document: DocumentMut,
}

impl UserConfig {
    /// Create a new config from a string.
    pub fn new(input: &str) -> anyhow::Result<Self> {
        let document: DocumentMut = input.parse().context("failed to parse config")?;

        Ok(Self { document })
    }

    /// Set the api key
    pub fn set_api_key(&mut self, api_key: String) -> anyhow::Result<()> {
        self.document
            .as_table_mut()
            .insert("api-key", api_key.into());
        Ok(())
    }
    
    /*
    pub fn get_api_key(&self) -> anyhow::Result<Option<()>> {
        self.document.as_table().get()
    }
    */

    /// Save the config to a path.
    pub async fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        let serialized = self.document.to_string();

        let temp_path = nd_util::with_push_extension(path, "tmp");
        tokio::fs::write(&temp_path, serialized.as_bytes()).await?;
        tokio::fs::rename(&temp_path, path).await?;

        Ok(())
    }
}
