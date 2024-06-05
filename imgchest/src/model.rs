mod post;
mod scraped_post;

pub use self::post::Image as PostImage;
pub use self::post::Post;
pub use self::scraped_post::FromElementError as InvalidScrapedPostImageError;
pub use self::scraped_post::FromHtmlError as InvalidScrapedPostError;
pub use self::scraped_post::Image as ScrapedPostImage;
pub use self::scraped_post::ScrapedPost;

/// The response to an api request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiResponse<T> {
    /// The payload
    pub data: T,
}
