use crate::Error;
use crate::Post;
use crate::PostImage;
use scraper::Html;

/// The client
#[derive(Debug, Clone)]
pub struct Client {
    /// The inner http client
    pub client: reqwest::Client,
}

impl Client {
    /// Make a new client
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("failed to build client");

        Self { client }
    }

    /// Get a post from a url
    pub async fn get_post(&self, url: &str) -> Result<Post, Error> {
        let text = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(tokio::task::spawn_blocking(move || {
            let html = Html::parse_document(text.as_str());
            Post::from_html(&html)
        })
        .await??)
    }

    /// Load extra images for a post
    pub async fn load_extra_images_for_post(&self, post: &Post) -> Result<Vec<PostImage>, Error> {
        let url = format!("https://imgchest.com/p/{}/loadAll", post.id);
        let text = self
            .client
            .post(url.as_str())
            .header("x-requested-with", "XMLHttpRequest")
            .form(&[("_token", post.token.as_str())])
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let images = tokio::task::spawn_blocking(move || {
            let html = Html::parse_fragment(&text);
            html.root_element()
                .children()
                .filter_map(scraper::ElementRef::wrap)
                .map(PostImage::from_element)
                .collect::<Result<Vec<_>, _>>()
        })
        .await??;

        Ok(images)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
