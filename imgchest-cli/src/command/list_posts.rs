use crate::UserConfig;
use anyhow::bail;
use anyhow::Context;
use imgchest::Url;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => bail!("unknown output format \"{input}\""),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum SortOrder {
    #[default]
    Popular,
    Old,
    New,
}

impl FromStr for SortOrder {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "popular" => Ok(Self::Popular),
            "old" => Ok(Self::Old),
            "new" => Ok(Self::New),
            _ => bail!("unknown sort order \"{input}\""),
        }
    }
}

impl From<SortOrder> for imgchest::SortOrder {
    fn from(sort: SortOrder) -> imgchest::SortOrder {
        match sort {
            SortOrder::Popular => Self::Popular,
            SortOrder::Old => Self::Old,
            SortOrder::New => Self::New,
        }
    }
}

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "list-posts",
    description = "list posts from various imgchest sources"
)]
pub struct Options {
    #[argh(
        option,
        long = "user",
        short = 'u',
        description = "only include posts by this user"
    )]
    user: Option<String>,

    #[argh(
        option,
        long = "page",
        description = "the page number to get",
        default = "1"
    )]
    page: u64,

    #[argh(
        option,
        long = "sort",
        short = 's',
        description = "how to sort posts",
        default = "Default::default()"
    )]
    sort: SortOrder,

    #[argh(
        switch,
        long = "profile",
        description = "whether to list posts for the current user"
    )]
    profile: bool,

    #[argh(
        option,
        long = "output-format",
        default = "Default::default()",
        description = "the output format"
    )]
    output_format: OutputFormat,
}

pub async fn exec(client: imgchest::Client, options: Options) -> anyhow::Result<()> {
    let config_dir = crate::util::get_config_dir().await?;

    let config_path = config_dir.join("config.toml");
    let config_str = crate::util::read_or_init_user_config_str(&config_path).await?;

    let user_config = UserConfig::new(&config_str)?;

    if let Some(cookies) = user_config.get_cookies()? {
        let mut cookie_store = client
            .get_cookie_store()
            .lock()
            .expect("cookie store is poisoned");

        for cookie in cookies.iter() {
            let cookie = imgchest::RawCookie::parse(cookie)
                .with_context(|| format!("failed to parse cookie \"{cookie}\""))?;
            let url = Url::parse("https://imgchest.com/")?;

            cookie_store.insert_raw(&cookie, &url)?;
        }
    }

    let mut builder = imgchest::ListPostsBuilder::new();
    builder
        .page(options.page)
        .sort(options.sort.into())
        .profile(options.profile);
    if let Some(user) = options.user {
        builder.username(user);
    }

    let posts = client
        .list_posts(builder)
        .await
        .context("failed to list posts")?;

    match options.output_format {
        OutputFormat::Human => output_human(&posts),
        OutputFormat::Json => output_json(&posts)?,
    }

    Ok(())
}

fn output_human(posts: &[imgchest::ListPostsPost]) {
    for post in posts.iter() {
        println!("Id: {}", post.id);
        println!("Title: {}", post.title);
        println!("Nsfw: {}", post.nsfw);
        println!("Score: {}", post.score);
        println!("Comments: {}", post.comments);
        println!("Views: {}", post.views);
        println!();
    }

    if posts.is_empty() {
        println!("No results");
    }
}

fn output_json(posts: &[imgchest::ListPostsPost]) -> anyhow::Result<()> {
    let stdout = std::io::stdout().lock();
    serde_json::to_writer(stdout, posts)?;
    Ok(())
}
