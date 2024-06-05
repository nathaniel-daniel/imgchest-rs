use std::num::NonZeroU32;
use time::OffsetDateTime;

/// An API post object
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Post {
    /// The post id
    pub id: Box<str>,

    /// The post title
    pub title: Box<str>,

    /// The post author's username
    pub username: Box<str>,

    /// The privacy of the post
    pub privacy: Box<str>,

    /// ?
    pub report_status: i32,

    /// The number of views
    pub views: u64,

    /// ?
    pub nsfw: i32,

    /// The number of images
    pub image_count: u64,

    /// The time this was created
    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,

    /// The files of this post
    pub images: Box<[File]>,

    /// The url to delete this post
    ///
    /// Only present if the current user owns this post.
    pub delete_url: Option<Box<str>>,
    // #[serde(flatten)]
    // extra: std::collections::HashMap<Box<str>, serde_json::Value>,
}

/// An API file of a post
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct File {
    /// The id of the image
    pub id: Box<str>,

    /// The image description
    #[serde(with = "maybe_box_str_empty_str_is_none")]
    pub description: Option<Box<str>>,

    /// The link to the image file
    pub link: Box<str>,

    /// The position of the image in the post.
    ///
    /// Starts at 1.
    pub position: NonZeroU32,

    /// The time this image was created.
    #[serde(with = "time::serde::iso8601")]
    pub created: OffsetDateTime,

    /// The original name of the image.
    ///
    /// Only present if the current user owns this image.
    pub original_name: Option<Box<str>>,
    // #[serde(flatten)]
    // extra: std::collections::HashMap<Box<str>, serde_json::Value>,
}

mod maybe_box_str_empty_str_is_none {
    use serde::Serialize;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<Box<str>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Box<str> = serde::Deserialize::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }

    pub(crate) fn serialize<S>(option: &Option<Box<str>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(value) = option {
            value.as_ref().serialize(serializer)
        } else {
            "".serialize(serializer)
        }
    }
}
