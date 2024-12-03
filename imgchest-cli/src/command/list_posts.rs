use anyhow::Context;

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
    let mut builder = imgchest::ListPostsBuilder::new();
    if let Some(user) = options.user {
        builder.username(user);
    }

    let posts = client
        .list_posts(builder)
        .await
        .context("failed to list posts")?;

    for post in posts {
        println!("Id: {}", post.id);
        println!("Title: {}", post.title);
        println!("Nsfw: {}", post.nsfw);
        println!("Score: {}", post.score);
        println!("Comments: {}", post.comments);
        println!("Views: {}", post.views);
        println!();
    }

    Ok(())
}
