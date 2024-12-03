use crate::UserConfig;
use anyhow::Context;
use imgchest::Url;

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
    if let Some(user) = options.user {
        builder.username(user);
    }

    let posts = client
        .list_posts(builder)
        .await
        .context("failed to list posts")?;

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

    Ok(())
}
