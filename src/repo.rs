use anyhow::{Context, Result};
use camino::Utf8Path;
use git2::{
    build::{CheckoutBuilder, RepoBuilder},
    Cred, FetchOptions, RemoteCallbacks, Repository,
};

pub fn clone_or_update(url: &str, target: &Utf8Path) -> Result<()> {
    if target.exists() && target.join(".git").exists() {
        update(url, target)?
    } else {
        clone(url, target)?
    };

    Ok(())
}

/// Update an already existing repository to the latest changes of the default head branch.
fn update(url: &str, target: &Utf8Path) -> Result<Repository> {
    let repo = Repository::open(target)?;

    {
        let mut remote = repo.remote_anonymous(url)?;
        let mut head = repo.head()?;
        let head_name = head.name().context("repo head is not valid UTF8")?;

        remote.fetch(&[head_name], Some(&mut create_fetch_opts()), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_head = fetch_head.resolve()?.peel_to_commit()?.id();

        head.set_target(fetch_head, "")?;
        repo.checkout_head(Some(
            CheckoutBuilder::new()
                .force()
                .remove_ignored(true)
                .remove_untracked(true),
        ))?;
    }

    Ok(repo)
}

/// Clone a new repo to the given output path or fail if it already exists.
fn clone(url: &str, target: &Utf8Path) -> Result<Repository> {
    let mut builder = RepoBuilder::new();
    builder.fetch_options(create_fetch_opts());

    builder.clone(url, target.as_std_path()).map_err(Into::into)
}

fn create_fetch_opts() -> FetchOptions<'static> {
    let callbacks = {
        let mut cb = RemoteCallbacks::new();
        cb.credentials(|_url, username, allowed_types| {
            if allowed_types.is_ssh_key() {
                if let Some(username) = username {
                    Cred::ssh_key_from_agent(username)
                } else {
                    Err(git2::Error::from_str(
                        "need username for SSH authentication",
                    ))
                }
            } else {
                Err(git2::Error::from_str(
                    "only SSH authentication is supported",
                ))
            }
        });
        cb
    };

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(callbacks);
    fo
}

/// Find the full repo name (including its owner) from a typical git URL as used on GitHub, GitLab
/// and other popular hosts.
///
/// The format one of (with the `.git` suffix being optional):
/// - `git@<host>:<owner>/<user>.git`
/// - `https://<host>/<owner>/<user>.git`
/// - `http://<host>/<owner>/<user>.git`
///
/// Therefore, to get the `<owner>/<user>` part, the prefix and suffix must be stripped. The final
/// string part is checked to contain only a single slash (`/`) to further validate the correctness
/// of the extracted name.
#[must_use]
pub fn find_repo_name(url: &str) -> Option<&str> {
    if url.starts_with("git@") {
        let name = url.split_once(':')?.1;
        Some(name.strip_suffix(".git").unwrap_or(name))
    } else if let Some(url) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    {
        let name = url.split_once('/')?.1;
        Some(name.strip_suffix(".git").unwrap_or(name))
    } else {
        None
    }
    .filter(|name| name.chars().filter(|&c| c == '/').count() == 1)
}

/// Initialize a new Git repository at the given location.
pub fn init(target: &Utf8Path) -> Result<()> {
    Repository::init(target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_git_url() {
        for input in &[
            "git@github.com:rust-lang/git2-rs",
            "git@github.com:rust-lang/git2-rs.git",
            "http://github.com/rust-lang/git2-rs",
            "http://github.com/rust-lang/git2-rs.git",
            "https://github.com/rust-lang/git2-rs",
            "https://github.com/rust-lang/git2-rs.git",
        ] {
            assert_eq!(Some("rust-lang/git2-rs"), find_repo_name(input));
        }
    }
}
