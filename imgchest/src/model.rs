mod scraped_post;

pub use self::scraped_post::FromElementError as InvalidScrapedPostImageError;
pub use self::scraped_post::FromHtmlError as InvalidScrapedPostError;
pub use self::scraped_post::Image as ScrapedPostImage;
pub use self::scraped_post::ScrapedPost;
