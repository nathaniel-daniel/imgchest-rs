use once_cell::sync::Lazy;
use scraper::Html;
use scraper::Selector;
use time::Date;
use time::OffsetDateTime;
use time::Time;

static APP_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("#app").unwrap());

/// An error that may occur while parsing a post
#[derive(Debug, thiserror::Error)]
pub enum FromHtmlError {
    #[error("missing {0}")]
    MissingElement(&'static str),

    #[error("missing attribute {0}")]
    MissingAttribute(&'static str),

    #[error("invalid data page")]
    InvalidDataPage(serde_json::Error),
}

/// A User
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrapedUser {
    /// The user's name
    pub name: Box<str>,

    /// The number of posts created by this user
    pub posts: u64,

    /// The number of comments created by this user
    pub comments: u64,

    /// The time this user was created.
    ///
    /// # Warning
    /// This is an estimate.
    pub created: OffsetDateTime,

    /// The number of views all posts made by this user have gotten.
    ///
    /// # Warning
    /// This is not a part of the real api struct.
    pub post_views: u64,

    /// The experience gained by the user?
    ///
    /// # Warning
    /// This is not a part of the real api struct.
    pub experience: u64,

    /// The number of favorites by the user.
    ///
    /// # Warning
    /// This is not a part of the real api struct.
    pub favorites: u64,
}

impl ScrapedUser {
    /// Parse this from html
    pub(crate) fn from_html(html: &Html) -> Result<Self, FromHtmlError> {
        // Implement:
        // JSON.parse(document.getElementById('app').getAttribute('data-page'))
        let app_element = html
            .select(&APP_SELECTOR)
            .next()
            .ok_or(FromHtmlError::MissingElement("app div"))?;
        let data_page_attr = app_element
            .attr("data-page")
            .ok_or(FromHtmlError::MissingAttribute("data-page"))?;
        let page_data: PageData =
            serde_json::from_str(data_page_attr).map_err(FromHtmlError::InvalidDataPage)?;

        Ok(Self {
            name: page_data.props.target_user.username,
            posts: page_data.props.target_user.post_count,
            comments: page_data.props.target_user.comment_count,
            created: OffsetDateTime::new_utc(
                page_data.props.target_user.created_at,
                Time::MIDNIGHT,
            ),

            post_views: page_data.props.target_user.post_views,
            experience: page_data.props.target_user.experience,
            favorites: page_data.props.target_user.favorite_count,
        })
    }
}

#[derive(Debug, serde::Deserialize)]
struct PageData {
    props: PageDataProps,
}

#[derive(Debug, serde::Deserialize)]
struct PageDataProps {
    #[serde(rename = "targetUser")]
    target_user: TargetUser,
}

#[derive(Debug, serde::Deserialize)]
struct TargetUser {
    username: Box<str>,
    post_count: u64,
    comment_count: u64,
    #[serde(with = "mdy_date")]
    created_at: Date,

    post_views: u64,
    experience: u64,
    favorite_count: u64,
}

time::serde::format_description!(mdy_date, Date, "[month]/[day]/[year]");
