mod post;
mod scraped_post;
mod user;

pub use self::post::File as PostFile;
pub use self::post::Post;
pub use self::scraped_post::File as ScrapedPostFile;
pub use self::scraped_post::FromElementError as InvalidScrapedPostFileError;
pub use self::scraped_post::FromHtmlError as InvalidScrapedPostError;
pub use self::scraped_post::ScrapedPost;
pub use self::user::User;

/// The response to an api request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiResponse<T> {
    /// The payload
    pub data: T,
}
