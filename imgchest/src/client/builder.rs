use crate::Error;
use crate::PostPrivacy;
use reqwest::multipart::Form;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T>>>;

pub(super) fn bool_to_str(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

/// A builder for creating a post.
///
/// This builder is for the low-level function.
#[derive(Debug)]
pub struct CreatePostBuilder {
    /// The title of the post.
    ///
    /// If specified, it must be at least 3 characters long.
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

impl CreatePostBuilder {
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
    ///
    /// It must be at least 3 characters long.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
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

    pub(super) fn try_clone(&self) -> Option<Self> {
        Some(Self {
            title: self.title.clone(),
            privacy: self.privacy,
            anonymous: self.anonymous,
            nsfw: self.nsfw,
            images: self
                .images
                .iter()
                .map(|image| image.try_clone())
                .collect::<Option<_>>()?,
        })
    }

    pub(super) async fn into_form(self) -> Result<Form, Error> {
        let mut form = Form::new();

        if let Some(title) = self.title {
            if title.len() < 3 {
                return Err(Error::TitleTooShort);
            }

            form = form.text("title", title);
        }

        if let Some(privacy) = self.privacy {
            form = form.text("privacy", privacy.as_str());
        }

        if let Some(anonymous) = self.anonymous {
            form = form.text("anonymous", bool_to_str(anonymous));
        }

        if let Some(nsfw) = self.nsfw {
            form = form.text("nsfw", bool_to_str(nsfw));
        }

        if self.images.is_empty() {
            return Err(Error::MissingImages);
        }

        for file in self.images {
            let body = file.body.into_body().await?;
            let part = reqwest::multipart::Part::stream(body).file_name(file.file_name);

            form = form.part("images[]", part);
        }

        Ok(form)
    }
}

impl Default for CreatePostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) enum UploadPostFileBody {
    Value(reqwest::Body),
    Func(Arc<dyn Fn() -> BoxFuture<std::io::Result<reqwest::Body>>>),
}

impl UploadPostFileBody {
    pub(super) async fn into_body(self) -> std::io::Result<reqwest::Body> {
        match self {
            UploadPostFileBody::Value(value) => Ok(value),
            UploadPostFileBody::Func(func) => func().await,
        }
    }

    pub(super) fn try_clone(&self) -> Option<Self> {
        match self {
            UploadPostFileBody::Value(_body) => None,
            UploadPostFileBody::Func(func) => Some(Self::Func(func.clone())),
        }
    }
}

/// A post file that is meant for uploading.
pub struct UploadPostFile {
    /// The file name
    pub(super) file_name: String,

    /// The file body
    pub(super) body: UploadPostFileBody,
}

impl UploadPostFile {
    /// Create this from a raw reqwest body.
    pub fn from_body(file_name: &str, body: reqwest::Body) -> Self {
        Self {
            file_name: file_name.into(),
            body: UploadPostFileBody::Value(body),
        }
    }

    /// Create this from bytes.
    pub fn from_bytes(file_name: &str, file_data: Vec<u8>) -> Self {
        Self::from_body(file_name, file_data.into())
    }

    /// Create this from a file.
    pub fn from_file(file_name: &str, file: tokio::fs::File) -> Self {
        let body = reqwest::Body::from(file);
        Self::from_body(file_name, body)
    }

    /// Create this from a file at the given path.
    pub async fn from_path<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        let file_name = path
            .file_name()
            .ok_or_else(|| std::io::Error::other("missing file name"))?
            .to_str()
            .ok_or_else(|| std::io::Error::other("file name is not valid unicode"))?;

        Ok(Self {
            file_name: file_name.into(),
            body: UploadPostFileBody::Func(Arc::new(move || {
                let path = path.clone();
                Box::pin(async move {
                    let file = tokio::fs::File::open(&path).await?;
                    Ok(reqwest::Body::from(file))
                })
            })),
        })
    }

    pub(super) fn try_clone(&self) -> Option<Self> {
        Some(Self {
            file_name: self.file_name.clone(),
            body: self.body.try_clone()?,
        })
    }
}

impl std::fmt::Debug for UploadPostFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("UploadPostFile")
            .field("file_name", &self.file_name)
            .finish()
    }
}

/// A builder for updating a post.
#[derive(Debug)]
pub struct UpdatePostBuilder {
    /// The title
    ///
    /// If specified, it must be at least 3 characters long.
    pub title: Option<String>,

    /// The post privacy
    pub privacy: Option<PostPrivacy>,

    /// Whether the post is nsfw
    pub nsfw: Option<bool>,
}

impl UpdatePostBuilder {
    /// Create an empty post update.
    pub fn new() -> Self {
        Self {
            title: None,
            privacy: None,
            nsfw: None,
        }
    }

    /// Update the title.
    ///
    /// It must be at least 3 characters long.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    /// Update the privacy.
    pub fn privacy(&mut self, privacy: PostPrivacy) -> &mut Self {
        self.privacy = Some(privacy);
        self
    }

    /// Update the nsfw flag.
    pub fn nsfw(&mut self, nsfw: bool) -> &mut Self {
        self.nsfw = Some(nsfw);
        self
    }
}

impl Default for UpdatePostBuilder {
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
