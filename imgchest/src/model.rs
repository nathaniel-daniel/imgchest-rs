mod list_posts;
mod post;
mod scraped_post;
mod scraped_user;
mod user;

pub use self::list_posts::ListPostsPost;
pub use self::post::File as PostFile;
pub use self::post::Post;
pub use self::post::Privacy as PostPrivacy;
pub use self::scraped_post::File as ScrapedPostFile;
pub use self::scraped_post::FromHtmlError as InvalidScrapedPostError;
pub use self::scraped_post::ScrapedPost;
pub use self::scraped_user::FromHtmlError as InvalidScrapedUserError;
pub use self::scraped_user::ScrapedUser;
pub use self::user::User;

/// A request for updating files in bulk.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiUpdateFilesBulkRequest {
    /// The payload
    pub data: Vec<FileUpdate>,
}

/// A file update as part of a bulk file update.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileUpdate {
    /// The file id
    pub id: String,

    /// The file description.
    ///
    /// Though the API docs seem to say that this field is nullable,
    /// it is not.
    pub description: String,
}

/// The response to an api request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiResponse<T> {
    /// The data payload
    pub data: T,
}

/// The response for when the api completed something
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiCompletedResponse {
    /// Whether the operation was successful.
    #[serde(with = "crate::serde::from_str_to_str")]
    pub success: bool,

    /// The operation message response.
    pub message: Option<Box<str>>,
}
