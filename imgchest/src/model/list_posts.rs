use std::collections::HashMap;

/// The post object from a list posts call.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ListPostsPost {
    /// The post id
    pub id: Box<str>,

    /// The post title
    pub title: Box<str>,

    /// The post slug
    pub slug: Box<str>,

    /// The post link
    pub link: Box<str>,

    /// Whether this is nsfw
    #[serde(with = "crate::serde::u8_to_bool")]
    pub nsfw: bool,

    /// The score of the post
    #[serde(with = "crate::serde::from_str_to_str")]
    pub score: i64,

    /// The number of comments on the post
    #[serde(with = "u64_or_str")]
    pub comments: u64,

    /// The number of views
    pub views: u64,

    /// The thumbnail
    pub thumbnail: Thumbnail,

    /// Extra key values
    #[serde(flatten)]
    pub extra: HashMap<Box<str>, serde_json::Value>,
}

/// The thumbnail object
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Thumbnail {
    /// The file id
    pub id: Box<str>,

    /// The file description
    pub description: Option<Box<str>>,

    /// A link to the thumbnail
    pub link: Box<str>,

    /// Extra key values
    #[serde(flatten)]
    pub extra: HashMap<Box<str>, serde_json::Value>,
}

mod u64_or_str {
    use serde::de::Visitor;

    struct U64OrStrVisitor;

    impl Visitor<'_> for U64OrStrVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a string of ascii digits or a u64")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            s.parse().map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(U64OrStrVisitor)
    }

    pub(crate) fn serialize<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(*value))
    }
}
