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

    /// Get cookies
    pub fn get_cookies(&self) -> anyhow::Result<Option<Vec<String>>> {
        let cookies = match self.document.as_table().get("cookies") {
            Some(cookies) => cookies,
            None => return Ok(None),
        };

        let cookies = cookies.as_value().context("cookies key is not a value")?;
        let cookies = cookies.as_array().context("cookies key is not an array")?;
        let cookies = cookies
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(|value| value.to_string())
                    .context("cookie array entry is not a string")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Some(cookies))
    }

    /// Save the config to a path.
    pub async fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        let serialized = self.document.to_string();

        let temp_path = nd_util::with_push_extension(path, "tmp");
        tokio::fs::write(&temp_path, serialized.as_bytes()).await?;
        tokio::fs::rename(&temp_path, path).await?;

        Ok(())
    }
}
