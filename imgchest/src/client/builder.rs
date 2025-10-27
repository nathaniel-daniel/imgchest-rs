use crate::PostPrivacy;
use std::path::Path;
use tokio_util::codec::BytesCodec;
use tokio_util::codec::FramedRead;

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
}

impl Default for CreatePostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A post file that is meant for uploading.
#[derive(Debug)]
pub struct UploadPostFile {
    /// The file name
    pub(super) file_name: String,

    /// The file body
    pub(super) body: reqwest::Body,
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
            .ok_or_else(|| std::io::Error::other("missing file name"))?
            .to_str()
            .ok_or_else(|| std::io::Error::other("file name is not valid unicode"))?;

        let file = tokio::fs::File::open(path).await?;

        Ok(Self::from_file(file_name, file))
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
