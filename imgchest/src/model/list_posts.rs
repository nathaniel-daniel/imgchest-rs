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
    #[serde(with = "int_or_str")]
    pub score: i64,

    /// The number of comments on the post
    #[serde(with = "int_or_str")]
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

mod int_or_str {
    use serde::de::Visitor;
    use std::marker::PhantomData;

    struct IntOrStrVisitor<T>(PhantomData<T>);

    impl<T> Visitor<'_> for IntOrStrVisitor<T>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
        T: TryFrom<u64>,
        <T as TryFrom<u64>>::Error: std::fmt::Display,
        T: TryFrom<i64>,
        <T as TryFrom<i64>>::Error: std::fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a stringified integer or an integer")
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
            value.try_into().map_err(E::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.try_into().map_err(E::custom)
        }
    }

    pub(crate) fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Display,
        T: TryFrom<u64>,
        <T as TryFrom<u64>>::Error: std::fmt::Display,
        T: TryFrom<i64>,
        <T as TryFrom<i64>>::Error: std::fmt::Display,
    {
        deserializer.deserialize_any(IntOrStrVisitor(PhantomData))
    }

    pub(crate) fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: itoa::Integer,
    {
        serializer.serialize_str(itoa::Buffer::new().format(*value))
    }
}
