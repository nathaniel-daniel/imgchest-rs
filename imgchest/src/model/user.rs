use jiff::Zoned;

/// The user
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct User {
    /// The user name
    pub name: Box<str>,

    /// The number of posts
    pub posts: u64,

    /// The number of comments
    pub comments: u64,

    /// The time this user was created
    #[serde(with = "crate::serde::iso8601_string")]
    pub created: Zoned,
    //#[serde(flatten)]
    //extra: std::collections::HashMap<Box<str>, serde_json::Value>,
}
