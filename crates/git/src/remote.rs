﻿use std::sync::LazyLock;

use derive_more::Deref;
use regex::Regex;
use url::Url;

/// The URL to a Git remote.
#[derive(Debug, PartialEq, Eq, Clone, Deref)]
pub struct RemoteUrl(Url);

static USERNAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-zA-Z\-_]+@").expect("Failed to create USERNAME_REGEX"));

impl std::str::FromStr for RemoteUrl {
    type Err = url::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if USERNAME_REGEX.is_match(input) {
            // Rewrite remote URLs like `git@github.com:user/repo.git` to `ssh://git@github.com/user/repo.git`
            let ssh_url = format!("ssh://{}", input.replacen(':', "/", 1));
            Ok(RemoteUrl(Url::parse(&ssh_url)?))
        } else {
            Ok(RemoteUrl(Url::parse(input)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parsing_valid_remote_urls() {
        let valid_urls = vec![
            (
                "https://github.com/octocat/CodeOrbit.git",
                "https",
                "github.com",
                "/octocat/CodeOrbit.git",
            ),
            (
                "git@github.com:octocat/CodeOrbit.git",
                "ssh",
                "github.com",
                "/octocat/CodeOrbit.git",
            ),
            (
                "org-000000@github.com:octocat/CodeOrbit.git",
                "ssh",
                "github.com",
                "/octocat/CodeOrbit.git",
            ),
            (
                "ssh://git@github.com/octocat/CodeOrbit.git",
                "ssh",
                "github.com",
                "/octocat/CodeOrbit.git",
            ),
            (
                "file:///path/to/local/CodeOrbit",
                "file",
                "",
                "/path/to/local/CodeOrbit",
            ),
        ];

        for (input, expected_scheme, expected_host, expected_path) in valid_urls {
            let parsed = input.parse::<RemoteUrl>().expect("failed to parse URL");
            let url = parsed.0;
            assert_eq!(
                url.scheme(),
                expected_scheme,
                "unexpected scheme for {input:?}",
            );
            assert_eq!(
                url.host_str().unwrap_or(""),
                expected_host,
                "unexpected host for {input:?}",
            );
            assert_eq!(url.path(), expected_path, "unexpected path for {input:?}");
        }
    }

    #[test]
    fn test_parsing_invalid_remote_urls() {
        let invalid_urls = vec!["not_a_url", "http://"];

        for url in invalid_urls {
            assert!(
                url.parse::<RemoteUrl>().is_err(),
                "expected \"{url}\" to not parse as a Git remote URL",
            );
        }
    }
}
