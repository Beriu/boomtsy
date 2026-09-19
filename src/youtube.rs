//! Turning whatever the user typed into something songbird can play.
//!
//! All of it goes through `yt-dlp`, which is an external binary rather than a
//! crate on purpose: YouTube rotates its player ciphers constantly, and yt-dlp
//! ships fixes within days. A vendored Rust scraper would be stale by the month.

use poise::serenity_prelude::UserId;
use reqwest::Client as HttpClient;
use songbird::input::{Compose, YoutubeDl};

use crate::{
    error::{BotError, UserError},
    song::Song,
};

/// A track that has been identified but not yet streamed.
pub struct Resolved {
    pub song: Song,
    /// Lazy: yt-dlp is run again at playback time, because the stream URLs it
    /// returns are short-lived and would expire while sitting in the queue.
    pub source: YoutubeDl<'static>,
}

/// Runs queries against yt-dlp. Holds the shared HTTP client songbird streams with.
#[derive(Clone)]
pub struct Resolver {
    http: HttpClient,
    /// Leaked once at startup so that every `YoutubeDl` it builds is `'static`.
    program: &'static str,
    extra_args: Vec<String>,
}

impl Resolver {
    pub fn new(http: HttpClient, program: &'static str, extra_args: Vec<String>) -> Self {
        Self {
            http,
            program,
            extra_args,
        }
    }

    /// Identify one playable track for `input`.
    pub async fn resolve(&self, input: &str, requested_by: UserId) -> Result<Resolved, BotError> {
        match classify(input) {
            Query::Link(url) => self.resolve_link(url, requested_by).await,
            Query::Search(terms) => self.resolve_search(terms, requested_by).await,
        }
    }

    async fn resolve_link(&self, url: String, requested_by: UserId) -> Result<Resolved, BotError> {
        let mut source = YoutubeDl::new_ytdl_like(self.program, self.http.clone(), url.clone())
            .user_args(self.extra_args.clone());
        let metadata = source.aux_metadata().await?;

        Ok(Resolved {
            song: Song::from_metadata(metadata, &url, requested_by),
            source,
        })
    }

    async fn resolve_search(
        &self,
        terms: String,
        requested_by: UserId,
    ) -> Result<Resolved, BotError> {
        let mut source =
            YoutubeDl::new_search_ytdl_like(self.program, self.http.clone(), terms.clone())
                .user_args(self.extra_args.clone());

        // Asking for exactly one match makes "yt-dlp found nothing" an empty
        // iterator rather than an error string we would have to pattern-match on.
        let top_match = source
            .search(Some(1))
            .await?
            .next()
            .ok_or(UserError::NoResults {
                query: terms.clone(),
            })?;

        Ok(Resolved {
            song: Song::from_metadata(top_match, &terms, requested_by),
            source,
        })
    }
}

/// What the user typed.
enum Query {
    /// Handed to yt-dlp verbatim. That covers every site it supports -- SoundCloud,
    /// Bandcamp, a bare .mp3 -- not just YouTube.
    Link(String),
    /// Becomes a YouTube search.
    Search(String),
}

fn classify(input: &str) -> Query {
    let trimmed = input.trim();

    let looks_like_a_link = trimmed
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty());

    if looks_like_a_link && !trimmed.contains(char::is_whitespace) {
        Query::Link(trimmed.to_owned())
    } else {
        Query::Search(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_link(input: &str) -> bool {
        matches!(classify(input), Query::Link(_))
    }

    #[test]
    fn treats_http_urls_as_links() {
        assert!(is_link("https://www.youtube.com/watch?v=K0HSD_i2DvA"));
        assert!(is_link("http://example.invalid/track.mp3"));
        assert!(is_link("  https://soundcloud.com/artist/track  "));
    }

    #[test]
    fn treats_words_as_a_search() {
        assert!(!is_link("daft punk around the world"));
        assert!(!is_link("nirvana"));
    }

    #[test]
    fn a_url_with_trailing_words_is_a_search_not_a_link() {
        // Passing this to yt-dlp verbatim would fail; searching for it at least
        // stands a chance of finding what they meant.
        assert!(!is_link("https://youtu.be/abc and then some"));
    }

    #[test]
    fn rejects_schemes_yt_dlp_should_not_be_handed() {
        assert!(!is_link("file:///etc/passwd"));
        assert!(!is_link("ftp://example.invalid/a.mp3"));
        assert!(!is_link("https://"));
    }

    #[test]
    fn trims_before_classifying() {
        let Query::Search(terms) = classify("  spacey query  ") else {
            panic!("expected a search");
        };
        assert_eq!(terms, "spacey query");
    }
}
