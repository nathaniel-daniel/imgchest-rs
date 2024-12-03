mod command;
mod user_config;
mod util;

pub use self::user_config::UserConfig;

#[derive(Debug, argh::FromArgs)]
#[argh(description = "a cli to interact with imgchest.com")]
struct Options {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, argh::FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Config(self::command::config::Options),
    Download(self::command::download::Options),
    Profile(self::command::profile::Options),
    ListPosts(self::command::list_posts::Options),
}

fn main() -> anyhow::Result<()> {
    let options = argh::from_env();
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    tokio_rt.block_on(async_main(options))
}

async fn async_main(options: Options) -> anyhow::Result<()> {
    let client = imgchest::Client::new();

    match options.subcommand {
        Subcommand::Config(options) => self::command::config::exec(options).await?,
        Subcommand::Download(options) => self::command::download::exec(client, options).await?,
        Subcommand::Profile(options) => self::command::profile::exec(client, options).await?,
        Subcommand::ListPosts(options) => self::command::list_posts::exec(client, options).await?,
    }

    Ok(())
}
