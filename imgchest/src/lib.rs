mod client;
mod model;

pub use self::client::Client;
pub use crate::model::InvalidPostError;
pub use crate::model::InvalidPostImageError;
pub use crate::model::Post;
pub use crate::model::PostImage;

/// The error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Reqwest error
    #[error("reqwest http error")]
    Reqwest(#[from] reqwest::Error),

    /// Failed to join tokio task
    #[error("failed to join tokio task")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// Failed to parse post
    #[error("invalid post")]
    InvalidPost(#[from] InvalidPostError),

    /// Failed to parse a post image
    #[error("invalid post image")]
    InvalidPostImage(#[from] InvalidPostImageError),
}

#[cfg(test)]
mod test {
    use super::*;

    const POST_URL: &str = "https://imgchest.com/p/3qe4gdvj4j2";
    const VIDEO_POST_URL: &str = "https://imgchest.com/p/pwl7lgepyx2";

    #[tokio::test]
    async fn get_post() {
        let client = Client::new();
        let post = client.get_post(POST_URL).await.expect("failed to get post");
        assert!(post.id == "3qe4gdvj4j2");
        assert!(post.title == "Donkey Kong - Video Game From The Mid 80's");
        assert!(post.username == "LunarLandr");
        // assert!(post.privacy == "public");
        // assert!(post.report_status == 1);
        assert!(post.views >= 198);
        // assert!(post.nsfw == 0);
        assert!(post.image_count == 4);
        // assert!(post.created == "2019-11-03T00:36:00.000000Z");

        assert!(post.images[0].id == "nw7w6cmlvye");
        assert!(post.images[0]
            .description
            .as_ref()
            .expect("missing description")
            .starts_with("Released in the arcades in 1981, Donkey Kong"));
        assert!(post.images[0].link == "https://cdn.imgchest.com/files/nw7w6cmlvye.png");
        assert!(post.images[0].video_link.is_none());

        assert!(post.images[1].id == "kwye3cpag4b");
        assert!(post.images[1].description.as_deref() == Some("amstrad - apple ii - atari - colecovision - c64 - msx\nnes - pc - vic-20 - spectrum - tI-99 4A - arcade"));
        assert!(post.images[1].link == "https://cdn.imgchest.com/files/kwye3cpag4b.png");
        assert!(post.images[1].video_link.is_none());

        assert!(post.images[2].id == "5g4z9c8ok72");
        assert!(post.images[2].description.is_none());
        assert!(post.images[2].link == "https://cdn.imgchest.com/files/5g4z9c8ok72.png");
        assert!(post.images[2].video_link.is_none());

        assert!(post.images[3].id == "we4gdcv5j4r");
        assert!(post.images[3].description.is_none());
        assert!(post.images[3].link == "https://cdn.imgchest.com/files/we4gdcv5j4r.jpg");
        assert!(post.images[3].video_link.is_none());

        dbg!(&post);
    }

    #[tokio::test]
    async fn get_video_post() {
        let client = Client::new();
        let post = client
            .get_post(VIDEO_POST_URL)
            .await
            .expect("failed to get post");

        assert!(post.id == "pwl7lgepyx2");
        assert!(post.title == "PDN AGIF Issue #1");
        assert!(post.username == "Jacob");
        assert!(post.views >= 2537);
        assert!(post.image_count == 1);

        assert!(post.images[0].id == "6yxkcz5ml7w");
        assert!(post.images[0].description.as_deref() == Some("Notice how inserting an AGIF is now supported, but does not want to be moved from its initial position."));
        assert!(post.images[0].link == "https://cdn.imgchest.com/files/6yxkcz5ml7w.gif");
        assert!(
            post.images[0].video_link.as_deref()
                == Some("https://cdn.imgchest.com/files/6yxkcz5ml7w.mp4")
        );

        dbg!(&post);
    }
}
