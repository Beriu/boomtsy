//! Turning whatever the user typed into something songbird can play.
//!
//! All of it goes through `yt-dlp`, which is an external binary rather than a
//! crate on purpose: YouTube rotates its player ciphers constantly, and yt-dlp
//! ships fixes within days. A vendored Rust scraper would be stale by the month.

use std::time::{Duration, Instant};

use poise::serenity_prelude::UserId;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use songbird::input::{Compose, YoutubeDl};
use tokio::process::Command;
use tracing::{info, warn};

use crate::{
    error::{BotError, UserError},
    song::Song,
};

/// How long yt-dlp gets before the command gives up on it.
///
/// yt-dlp enforces no deadline of its own, and when YouTube makes it solve a JS
/// challenge without a usable solver it can grind for minutes. Left unbounded
/// that leaves the slash command deferred until Discord expires the token, so
/// the person who typed it watches "thinking..." forever and never learns that
/// anything went wrong. Better to say so and let them retry.
///
/// Note that this abandons the yt-dlp process rather than killing it: songbird
/// spawns it without `kill_on_drop`, so a stalled one runs to completion in the
/// background. It exits on its own; it just stops being anyone's problem.
const RESOLVE_DEADLINE: Duration = Duration::from_secs(45);

/// How many search results to look through.
///
/// There has to be room to look past the first: for a bare artist name YouTube
/// returns that artist's channel as the top hit.
const SEARCH_RESULTS: usize = 5;

/// yt-dlp's extractor key for a single video.
///
/// Anything else a search returns -- `YoutubeTab`, which covers channels and
/// playlists -- has to be skipped rather than played. Handing yt-dlp a channel
/// makes it walk the entire catalogue behind it: minutes of work that ends in a
/// track nobody asked for, and the single worst hang this bot can produce.
const VIDEO_EXTRACTOR: &str = "Youtube";

/// A resolve slower than this still succeeds, but says something is wrong with
/// the extraction path -- a healthy lookup is a few seconds.
const SLOW_RESOLVE: Duration = Duration::from_secs(10);

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
    ///
    /// Every attempt is timed and logged. songbird gives yt-dlp's stderr to no
    /// one, so without this the only evidence of a YouTube extraction problem is
    /// a user saying the bot feels slow.
    pub async fn resolve(&self, input: &str, requested_by: UserId) -> Result<Resolved, BotError> {
        let attempt = async {
            match classify(input) {
                Query::Link(url) => self.resolve_link(url, requested_by).await,
                Query::Search(terms) => self.resolve_search(terms, requested_by).await,
            }
        };

        let started = Instant::now();
        let outcome = tokio::time::timeout(RESOLVE_DEADLINE, attempt).await;
        let took = started.elapsed();

        match &outcome {
            Ok(Ok(resolved)) if took >= SLOW_RESOLVE => warn!(
                seconds = took.as_secs_f32(),
                track = resolved.song.title,
                "yt-dlp was slow: YouTube is most likely making it solve a JS \
                 challenge. Check that deno is on PATH and that YTDLP_EXTRA_ARGS \
                 carries --remote-components ejs:github"
            ),
            Ok(Ok(resolved)) => info!(
                seconds = took.as_secs_f32(),
                track = resolved.song.title,
                "resolved"
            ),
            Ok(Err(failure)) => warn!(
                seconds = took.as_secs_f32(),
                %failure,
                query = input,
                "resolve failed"
            ),
            Err(_elapsed) => warn!(
                seconds = took.as_secs_f32(),
                query = input,
                "yt-dlp passed the deadline and was abandoned"
            ),
        }

        outcome.map_err(|_elapsed| UserError::ResolveTimedOut)?
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

    /// Searching happens in two steps rather than through songbird's own search,
    /// which would hand yt-dlp `ytsearch1:<terms>` and extract whatever came back
    /// -- channel or not. Worse, songbird re-runs that same search when playback
    /// actually starts, so a bad top hit stalls twice. Picking a concrete video
    /// here means the thing that eventually plays is a plain URL.
    async fn resolve_search(
        &self,
        terms: String,
        requested_by: UserId,
    ) -> Result<Resolved, BotError> {
        let url = self.first_video_for(&terms).await?;
        self.resolve_link(url, requested_by).await
    }

    /// The URL of the first search result that is an actual video.
    async fn first_video_for(&self, terms: &str) -> Result<String, BotError> {
        // --flat-playlist lists what a search found without extracting any of it,
        // which is both far faster and the whole point: extraction is what hangs.
        let listing = Command::new(self.program)
            .args(&self.extra_args)
            .args([
                "--flat-playlist",
                "--dump-single-json",
                &format!("ytsearch{SEARCH_RESULTS}:{terms}"),
            ])
            .output()
            .await
            .map_err(|failure| {
                BotError::Search(format!("could not run {}: {failure}", self.program))
            })?;

        if !listing.status.success() {
            let complaint = String::from_utf8_lossy(&listing.stderr);
            return Err(BotError::Search(format!(
                "{} exited with {}: {}",
                self.program,
                listing.status,
                complaint.trim().chars().take(300).collect::<String>()
            )));
        }

        let found: FlatSearch = serde_json::from_slice(&listing.stdout)
            .map_err(|failure| BotError::Search(format!("unreadable search output: {failure}")))?;

        first_video(found.entries).ok_or_else(|| {
            BotError::from(UserError::NoResults {
                query: terms.to_owned(),
            })
        })
    }
}

/// Just enough of `--flat-playlist --dump-single-json` to choose a result.
#[derive(Deserialize)]
struct FlatSearch {
    #[serde(default)]
    entries: Vec<FlatEntry>,
}

#[derive(Deserialize)]
struct FlatEntry {
    /// Which extractor yt-dlp would use. See [`VIDEO_EXTRACTOR`].
    #[serde(default)]
    ie_key: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

/// Pick the first entry that is a playable video, discarding the channels and
/// playlists a search mixes in.
fn first_video(entries: Vec<FlatEntry>) -> Option<String> {
    entries
        .into_iter()
        .filter(|entry| entry.ie_key.as_deref() == Some(VIDEO_EXTRACTOR))
        .find_map(|entry| entry.url)
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

    fn entry(ie_key: &str, url: &str) -> FlatEntry {
        FlatEntry {
            ie_key: Some(ie_key.to_owned()),
            url: Some(url.to_owned()),
        }
    }

    /// The exact shape that hung the bot: searching an artist by name puts their
    /// channel first, and extracting a channel walks its whole catalogue.
    #[test]
    fn skips_the_channel_that_a_bare_artist_name_returns_first() {
        let results = vec![
            entry(
                "YoutubeTab",
                "https://www.youtube.com/channel/UCJhe07czqzg4Uzpjv",
            ),
            entry("Youtube", "https://www.youtube.com/watch?v=DWaB4PXCwFU"),
            entry("Youtube", "https://www.youtube.com/watch?v=UkI4KejmSfY"),
        ];

        assert_eq!(
            first_video(results).as_deref(),
            Some("https://www.youtube.com/watch?v=DWaB4PXCwFU")
        );
    }

    #[test]
    fn takes_the_top_hit_when_it_is_already_a_video() {
        let results = vec![entry("Youtube", "https://www.youtube.com/watch?v=aaa")];

        assert_eq!(
            first_video(results).as_deref(),
            Some("https://www.youtube.com/watch?v=aaa")
        );
    }

    #[test]
    fn reports_nothing_when_a_search_returns_no_videos_at_all() {
        let results = vec![
            entry("YoutubeTab", "https://www.youtube.com/channel/abc"),
            entry("YoutubeTab", "https://www.youtube.com/playlist?list=xyz"),
        ];

        assert!(first_video(results).is_none());
        assert!(first_video(Vec::new()).is_none());
    }

    #[test]
    fn ignores_a_video_entry_that_carries_no_url() {
        let results = vec![
            FlatEntry {
                ie_key: Some("Youtube".to_owned()),
                url: None,
            },
            entry("Youtube", "https://www.youtube.com/watch?v=good"),
        ];

        assert_eq!(
            first_video(results).as_deref(),
            Some("https://www.youtube.com/watch?v=good")
        );
    }

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
