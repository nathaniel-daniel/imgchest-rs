use crate::ApiResponse;
use crate::Error;
use crate::Post;
use crate::PostFile;
use crate::ScrapedPost;
use crate::ScrapedPostFile;
use crate::User;
use reqwest::header::AUTHORIZATION;
use scraper::Html;
use std::sync::Arc;

/// The client
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client
    pub client: reqwest::Client,

    /// Inner client state
    state: Arc<ClientState>,
}

impl Client {
    /// Make a new client
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("failed to build client");
        let state = Arc::new(ClientState {
            token: std::sync::RwLock::new(None),
        });

        Self { client, state }
    }

    /// Scrape a post from a url.
    ///
    /// # Authorization
    /// This function does NOT require the use of a token.
    pub async fn get_scraped_post(&self, url: &str) -> Result<ScrapedPost, Error> {
        let text = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let post = tokio::task::spawn_blocking(move || {
            let html = Html::parse_document(text.as_str());
            ScrapedPost::from_html(&html)
        })
        .await??;

        Ok(post)
    }

    /// Load extra files for a scraped post.
    ///
    /// # Authorization
    /// This function does NOT require the use of a token.
    pub async fn load_extra_files_for_scraped_post(
        &self,
        post: &ScrapedPost,
    ) -> Result<Vec<ScrapedPostFile>, Error> {
        let id = &post.id;
        let url = format!("https://imgchest.com/p/{id}/loadAll");
        let text = self
            .client
            .post(url.as_str())
            .header("x-requested-with", "XMLHttpRequest")
            .form(&[("_token", &*post.token)])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let images = tokio::task::spawn_blocking(move || {
            let html = Html::parse_fragment(&text);
            html.root_element()
                .children()
                .filter_map(scraper::ElementRef::wrap)
                .map(ScrapedPostFile::from_element)
                .collect::<Result<Vec<_>, _>>()
        })
        .await??;

        Ok(images)
    }

    /// Set the token to use for future requests.
    ///
    /// This allows the use of functions that require authorization.
    pub fn set_token<T>(&self, token: T)
    where
        T: AsRef<str>,
    {
        *self
            .state
            .token
            .write()
            .unwrap_or_else(|error| error.into_inner()) = Some(token.as_ref().into());
    }

    /// Get the current token.
    fn get_token(&self) -> Option<Arc<str>> {
        self.state
            .token
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    /// Get a post by id.
    ///
    /// # Authorization
    /// This function does REQUIRES a token.
    pub async fn get_post(&self, id: &str) -> Result<Post, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("https://api.imgchest.com/v1/post/{id}");

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let post: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(post.data)
    }

    /// Get a user by username.
    ///
    /// # Authorization
    /// This function does REQUIRES a token.
    pub async fn get_user(&self, username: &str) -> Result<User, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("https://api.imgchest.com/v1/user/{username}");

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let user: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(user.data)
    }

    /// Get a file by id.
    ///
    /// Currently, this is implemented according to the API spec,
    /// but the API will always return no data for some reason.
    /// It is likely that this endpoint is disabled.
    /// As a result, this function is currently useless.
    ///
    /// # Authorization
    /// This function does REQUIRES a token.
    pub async fn get_file(&self, id: &str) -> Result<PostFile, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("https://api.imgchest.com/v1/file/{id}");

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let file: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(file.data)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ClientState {
    token: std::sync::RwLock<Option<Arc<str>>>,
}
