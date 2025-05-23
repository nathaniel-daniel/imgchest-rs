mod builder;

pub use self::builder::CreatePostBuilder;
pub use self::builder::UpdatePostBuilder;
pub use self::builder::UploadPostFile;
use crate::ApiCompletedResponse;
use crate::ApiResponse;
use crate::ApiUpdateFilesBulkRequest;
use crate::Error;
use crate::FileUpdate;
use crate::ListPostsPost;
use crate::Post;
use crate::PostFile;
use crate::ScrapedPost;
use crate::ScrapedUser;
use crate::User;
use reqwest::header::AUTHORIZATION;
use reqwest::multipart::Form;
use reqwest::Url;
use reqwest_cookie_store::CookieStore;
use reqwest_cookie_store::CookieStoreMutex;
use scraper::Html;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

const REQUESTS_PER_MINUTE: u8 = 60;
const ONE_MINUTE: Duration = Duration::from_secs(60);
const API_BASE: &str = "https://api.imgchest.com";

/// A builder for listing posts
#[derive(Debug, Clone)]
pub struct ListPostsBuilder {
    /// How posts should be sorted.
    ///
    /// Defaults to popular.
    pub sort: SortOrder,

    /// The page to get.
    ///
    /// Starts at 1.
    pub page: u64,

    /// The username to filter posts by.
    pub username: Option<String>,

    /// Whether to list posts from the current user.
    pub profile: bool,
}

impl ListPostsBuilder {
    /// Make a new builder
    pub fn new() -> Self {
        Self {
            sort: SortOrder::Popular,
            page: 1,
            username: None,
            profile: false,
        }
    }

    /// Set how posts should be sorted.
    ///
    /// Defaults to popular.
    pub fn sort(&mut self, sort: SortOrder) -> &mut Self {
        self.sort = sort;
        self
    }

    /// Set the page to get.
    ///
    /// Starts at 1.
    pub fn page(&mut self, page: u64) -> &mut Self {
        self.page = page;
        self
    }

    /// Set the username to filter by.
    pub fn username(&mut self, username: String) -> &mut Self {
        self.username = Some(username);
        self
    }

    /// Set whether to list posts from the current user.
    pub fn profile(&mut self, profile: bool) -> &mut Self {
        self.profile = profile;
        self
    }
}

impl Default for ListPostsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort order
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SortOrder {
    /// Sort by the most popular posts
    Popular,

    /// Sort by the newest posts
    New,

    /// Sort by the oldest posts
    Old,
}

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
        let state = Arc::new(ClientState::new());

        let client = reqwest::Client::builder()
            .cookie_provider(state.cookie_store.clone())
            .build()
            .expect("failed to build client");

        Self { client, state }
    }

    /// Scrape a post from a post id.
    ///
    /// # Authorization
    /// This function does NOT require the use of a token.
    ///
    /// # Warning
    /// This is a scraping-based function.
    pub async fn get_scraped_post(&self, id: &str) -> Result<ScrapedPost, Error> {
        let url = format!("https://imgchest.com/p/{id}");
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

    /// Scrape a user from a username.
    ///
    /// # Authorization
    /// This function does NOT require the use of a token.
    ///
    /// # Warning
    /// This is a scraping-based function.
    pub async fn get_scraped_user(&self, name: &str) -> Result<ScrapedUser, Error> {
        let url = format!("https://imgchest.com/u/{name}");
        let text = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let user = tokio::task::spawn_blocking(move || {
            let html = Html::parse_document(text.as_str());
            ScrapedUser::from_html(&html)
        })
        .await??;

        Ok(user)
    }

    /// List posts from various sources.
    ///
    /// # Authorization
    /// This function does NOT require the use of a token.
    ///
    /// # Warning
    /// This api call is undocumented.
    pub async fn list_posts(&self, builder: ListPostsBuilder) -> Result<Vec<ListPostsPost>, Error> {
        let mut url = Url::parse("https://imgchest.com/api/posts").unwrap();
        {
            let mut query_pairs = url.query_pairs_mut();

            let sort_str = match builder.sort {
                SortOrder::Popular => "popular",
                SortOrder::New => "new",
                SortOrder::Old => "old",
            };
            query_pairs.append_pair("sort", sort_str);

            query_pairs.append_pair("page", itoa::Buffer::new().format(builder.page));

            if let Some(username) = builder.username.as_deref() {
                query_pairs.append_pair("username", username);
            }

            if builder.profile {
                query_pairs.append_pair("profile", "true");
            }
        }

        let response = self.client.get(url.as_str()).send().await?;

        let posts: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(posts.data)
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

    /// Get the cookie store.
    pub fn get_cookie_store(&self) -> &Arc<CookieStoreMutex> {
        &self.state.cookie_store
    }

    /// Get a post by id.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn get_post(&self, id: &str) -> Result<Post, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post/{id}");

        self.state.ratelimit().await;

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let post: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(post.data)
    }

    /// Create a post.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn create_post(&self, data: CreatePostBuilder) -> Result<Post, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post");

        let mut form = Form::new();

        if let Some(title) = data.title {
            if title.len() < 3 {
                return Err(Error::TitleTooShort);
            }

            form = form.text("title", title);
        }

        if let Some(privacy) = data.privacy {
            form = form.text("privacy", privacy.as_str());
        }

        if let Some(anonymous) = data.anonymous {
            form = form.text("anonymous", bool_to_str(anonymous));
        }

        if let Some(nsfw) = data.nsfw {
            form = form.text("nsfw", bool_to_str(nsfw));
        }

        if data.images.is_empty() {
            return Err(Error::MissingImages);
        }

        for file in data.images {
            let part = reqwest::multipart::Part::stream(file.body).file_name(file.file_name);

            form = form.part("images[]", part);
        }

        self.state.ratelimit().await;

        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .multipart(form)
            .send()
            .await?;

        let post: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(post.data)
    }

    /// Update a post.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn update_post(&self, id: &str, data: UpdatePostBuilder) -> Result<Post, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post/{id}");

        let mut form = Vec::new();

        if let Some(title) = data.title.as_ref() {
            if title.len() < 3 {
                return Err(Error::TitleTooShort);
            }

            form.push(("title", title.as_str()));
        }

        if let Some(privacy) = data.privacy {
            form.push(("privacy", privacy.as_str()));
        }

        if let Some(nsfw) = data.nsfw {
            form.push(("nsfw", bool_to_str(nsfw)));
        }

        self.state.ratelimit().await;

        // Not using a multipart form here is intended.
        // Even though we use a multipart form for creating a post,
        // the server will silently ignore requests that aren't form-urlencoded.
        let response = self
            .client
            .patch(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .form(&form)
            .send()
            .await?;

        let post: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(post.data)
    }

    /// Delete a post.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn delete_post(&self, id: &str) -> Result<(), Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post/{id}");

        self.state.ratelimit().await;

        let response = self
            .client
            .delete(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let response: ApiCompletedResponse = response.error_for_status()?.json().await?;
        if !response.success {
            return Err(Error::ApiOperationFailed);
        }

        Ok(())
    }

    /// Favorite or unfavorite a post.
    ///
    /// # Returns
    /// Returns true if the favorite was added.
    /// Returns false if the favorite was removed.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn favorite_post(&self, id: &str) -> Result<bool, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post/{id}/favorite");

        self.state.ratelimit().await;

        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let response: ApiCompletedResponse = response.error_for_status()?.json().await?;
        if !response.success {
            return Err(Error::ApiOperationFailed);
        }

        let message = response.message.ok_or(Error::ApiResponseMissingMessage)?;
        match &*message {
            "Favorite added." => Ok(true),
            "Favorite removed." => Ok(false),
            _ => Err(Error::ApiResponseUnknownMessage { message }),
        }
    }

    /// Add images to a post.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn add_post_images<I>(&self, id: &str, images: I) -> Result<Post, Error>
    where
        I: IntoIterator<Item = UploadPostFile>,
    {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/post/{id}/add");

        let mut form = Form::new();

        let mut num_images = 0;
        for file in images {
            let part = reqwest::multipart::Part::stream(file.body).file_name(file.file_name);

            form = form.part("images[]", part);
            num_images += 1;
        }

        if num_images == 0 {
            return Err(Error::MissingImages);
        }

        self.state.ratelimit().await;

        let response = self
            .client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .multipart(form)
            .send()
            .await?;

        let post: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(post.data)
    }

    /// Get a user by username.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn get_user(&self, username: &str) -> Result<User, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/user/{username}");

        self.state.ratelimit().await;

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
    /// This function REQUIRES a token.
    pub async fn get_file(&self, id: &str) -> Result<PostFile, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/file/{id}");

        self.state.ratelimit().await;

        let response = self
            .client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let file: ApiResponse<_> = response.error_for_status()?.json().await?;

        Ok(file.data)
    }

    /// Update a file.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn update_file(&self, id: &str, description: &str) -> Result<(), Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/file/{id}");

        if description.is_empty() {
            return Err(Error::MissingDescription);
        }

        self.state.ratelimit().await;

        let response = self
            .client
            .patch(url)
            .form(&[("description", description)])
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let response: ApiCompletedResponse = response.error_for_status()?.json().await?;
        if !response.success {
            return Err(Error::ApiOperationFailed);
        }

        Ok(())
    }

    /// Delete a file.
    ///
    /// # Authorization
    /// This function REQUIRES a token.
    pub async fn delete_file(&self, id: &str) -> Result<(), Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/file/{id}");

        self.state.ratelimit().await;

        let response = self
            .client
            .delete(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await?;

        let response: ApiCompletedResponse = response.error_for_status()?.json().await?;
        if !response.success {
            return Err(Error::ApiOperationFailed);
        }

        Ok(())
    }

    /// Update files in bulk.
    pub async fn update_files_bulk<I>(&self, files: I) -> Result<Vec<PostFile>, Error>
    where
        I: IntoIterator<Item = FileUpdate>,
    {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = format!("{API_BASE}/v1/files");

        let data = files
            .into_iter()
            .map(|file| {
                if file.description.is_empty() {
                    return Err(Error::MissingDescription);
                }
                Ok(file)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let data = ApiUpdateFilesBulkRequest { data };

        self.state.ratelimit().await;

        let response = self
            .client
            .patch(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .json(&data)
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
    ratelimit_data: std::sync::Mutex<(Instant, u8)>,

    cookie_store: Arc<CookieStoreMutex>,
}

impl ClientState {
    fn new() -> Self {
        let token = std::sync::RwLock::new(None);

        let now = Instant::now();
        let ratelimit_data = std::sync::Mutex::new((now, REQUESTS_PER_MINUTE));

        let cookie_store = CookieStore::new(None);
        let cookie_store = CookieStoreMutex::new(cookie_store);
        let cookie_store = Arc::new(cookie_store);

        Self {
            token,
            ratelimit_data,

            cookie_store,
        }
    }

    async fn ratelimit(&self) {
        loop {
            let sleep_duration = {
                let mut ratelimit_data = self
                    .ratelimit_data
                    .lock()
                    .expect("ratelimit mutex poisoned");
                let (ref mut last_refreshed, ref mut remaining_requests) = &mut *ratelimit_data;

                // Refresh the number of requests each minute.
                if last_refreshed.elapsed() >= ONE_MINUTE {
                    *last_refreshed = Instant::now();
                    *remaining_requests = REQUESTS_PER_MINUTE;
                }

                // If we are allowed to make a request now, make it.
                if *remaining_requests > 0 {
                    *remaining_requests -= 1;
                    return;
                }

                // Otherwise, sleep until the next refresh and try again.
                ONE_MINUTE.saturating_sub(last_refreshed.elapsed())
            };
            tokio::time::sleep(sleep_duration).await;
        }
    }
}

fn bool_to_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}
