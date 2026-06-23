use crate::UserConfig;
use anyhow::Context;
use imgchest::Url;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, clap::ValueEnum)]
pub enum SortOrder {
    #[default]
    Popular,
    Old,
    New,
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

#[derive(Debug, Clone, clap::Parser)]
#[command(about = "List posts from various imgchest sources")]
pub struct Options {
    #[arg(long = "user", short = 'u', help = "Only include posts by this user")]
    user: Option<String>,

    #[arg(long = "page", help = "The page number to get", default_value = "1")]
    page: u64,

    #[arg(
        value_enum,
        long = "sort",
        short = 's',
        help = "How to sort posts",
        default_value = "popular"
    )]
    sort: SortOrder,

    #[arg(long = "profile", help = "Whether to list posts for the current user")]
    profile: bool,

    #[arg(
        long = "output-format",
        default_value = "human",
        help = "The output format"
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
