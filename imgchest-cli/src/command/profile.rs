use anyhow::ensure;
use anyhow::Context;
use url::Url;

#[derive(Debug, argh::FromArgs)]
#[argh(
    subcommand,
    name = "profile",
    description = "get profile information for a user"
)]
pub struct Options {
    #[argh(positional, description = "the user to fetch profile data for")]
    pub user: String,
}

pub async fn exec(client: imgchest::Client, options: Options) -> anyhow::Result<()> {
    let user = extract_user(options.user).context("failed to extract user")?;

    let user = client
        .get_scraped_user(&user)
        .await
        .context("failed to scrape user")?;

    let created_date = user.created.date();

    println!("Name: {}", user.name);
    println!(
        "Joined: {}/{}/{}",
        created_date.month() as u8,
        created_date.day(),
        created_date.year()
    );
    println!("XP: {}", PrettyFormatU64(user.experience));
    println!("Posts: {}", user.posts);
    println!("Comments: {}", user.comments);
    println!("Favorites: {}", user.favorites);
    println!("Post Views: {}", PrettyFormatU64(user.post_views));

    Ok(())
}

fn extract_user(input: String) -> anyhow::Result<String> {
    match Url::parse(input.as_str()) {
        Ok(url) => {
            // Validate url.
            // We try to match https://imgchest.com/u/{user}
            let host_str = url.host_str();
            ensure!(
                host_str == Some("imgchest.com"),
                "url hast unknown host {host_str:?}"
            );

            let mut path = url.path_segments().context("url missing path")?;
            ensure!(path.next() == Some("u"));

            let user = path.next().context("url missing user path segment")?;

            Ok(user.to_string())
        }
        Err(_error) => {
            // Assume this is a user.
            Ok(input)
        }
    }
}

struct PrettyFormatU64(pub u64);

impl std::fmt::Display for PrettyFormatU64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0 == 0 {
            return self.0.fmt(f);
        }

        let num_digits = self.0.ilog10() + 1;
        for digit in (0..num_digits).rev() {
            let n = (self.0 / (10_u64.pow(digit))) % 10;
            write!(f, "{n}")?;

            if digit != 0 && digit % 3 == 0 {
                write!(f, ",")?;
            }
        }

        Ok(())
    }
}
