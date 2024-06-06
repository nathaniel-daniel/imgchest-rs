use crate::ApiResponse;
use crate::Error;
use crate::Post;
use crate::PostFile;
use crate::PostPrivacy;
use crate::ScrapedPost;
use crate::ScrapedPostFile;
use crate::User;
use reqwest::header::AUTHORIZATION;
use reqwest::multipart::Form;
use scraper::Html;
use std::path::Path;
use std::sync::Arc;
use tokio_util::codec::{BytesCodec, FramedRead};

/// A builder for creating a post.
///
/// This builder is for the low-level function.
#[derive(Debug)]
pub struct CreatePostRawBuilder {
    /// The title of the post.
    pub title: Option<String>,

    /// The post privacy.
    ///
    /// Defaults to hidden.
    pub privacy: Option<PostPrivacy>,

    /// Whether the post should be tied to the user.
    pub anonymous: Option<bool>,

    /// Whether this post is nsfw.
    pub nsfw: Option<bool>,

    /// The images of the post
    pub images: Vec<UploadPostFile>,
}

impl CreatePostRawBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            title: None,
            privacy: None,
            anonymous: None,
            nsfw: None,
            images: Vec::new(),
        }
    }

    /// Set the title.
    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Set the post privacy.
    ///
    /// Defaults to hidden.
    pub fn privacy(&mut self, privacy: PostPrivacy) -> &mut Self {
        self.privacy = Some(privacy);
        self
    }

    /// Set whether this post should be anonymous.
    pub fn anonymous(&mut self, anonymous: bool) -> &mut Self {
        self.anonymous = Some(anonymous);
        self
    }

    /// Set whether this post is nsfw.
    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.nsfw = Some(nsfw);
        self
    }

    /// Add a new image to this post.
    pub fn image(&mut self, file: UploadPostFile) -> &mut Self {
        self.images.push(file);
        self
    }
}

impl Default for CreatePostRawBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A post file that is meant for uploading.
#[derive(Debug)]
pub struct UploadPostFile {
    /// The file name
    file_name: String,

    /// The file body
    body: reqwest::Body,
}

impl UploadPostFile {
    /// Create this from a raw reqwest body.
    pub fn from_body(file_name: &str, body: reqwest::Body) -> Self {
        Self {
            file_name: file_name.into(),
            body,
        }
    }

    /// Create this from bytes.
    pub fn from_bytes(file_name: &str, file_data: Vec<u8>) -> Self {
        Self::from_body(file_name, file_data.into())
    }

    /// Create this from a file.
    pub fn from_file(file_name: &str, file: tokio::fs::File) -> Self {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = reqwest::Body::wrap_stream(stream);

        Self::from_body(file_name, body)
    }

    /// Create this from a file at the given path.
    pub async fn from_path<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let file_name = path
            .file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "missing file name"))?
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "file name is not valid unicode")
            })?;

        let file = tokio::fs::File::open(path).await?;

        Ok(Self::from_file(file_name, file))
    }
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

    /// Create a post.
    ///
    /// # Authorization
    /// This function does REQUIRES a token.
    pub async fn create_post_raw(&self, data: CreatePostRawBuilder) -> Result<Post, Error> {
        let token = self.get_token().ok_or(Error::MissingToken)?;
        let url = "https://api.imgchest.com/v1/post";

        let mut form = Form::new();

        if let Some(title) = data.title {
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

fn bool_to_str(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}
