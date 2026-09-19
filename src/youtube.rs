//! Turning whatever the user typed into something songbird can play.
//!
//! All of it goes through `yt-dlp`, which is an external binary rather than a
//! crate on purpose: YouTube rotates its player ciphers constantly, and yt-dlp
//! ships fixes within days. A vendored Rust scraper would be stale by the month.

use std::time::{Duration, Instant};

use poise::serenity_prelude::UserId;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use songbird::input::{AuxMetadata, Compose, YoutubeDl};
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

/// Most tracks taken from one playlist.
///
/// A channel's uploads playlist can run to several hundred entries, and nobody
/// who pastes one wants all of them queued.
pub const PLAYLIST_LIMIT: usize = 100;

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

/// Whether a link that also names a playlist should be expanded.
///
/// YouTube's share button hands out `watch?v=...&list=...` for a track inside a
/// playlist, so the URL alone cannot say which the person meant. The default is
/// the single track, because queueing a hundred unwanted songs over whatever is
/// playing is far more disruptive than queueing one too few.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaylistHandling {
    SingleTrack,
    Expand,
}

impl PlaylistHandling {
    pub fn from_flag(expand: Option<bool>) -> Self {
        if expand.unwrap_or(false) {
            Self::Expand
        } else {
            Self::SingleTrack
        }
    }
}

/// What a query turned out to be.
pub enum Resolution {
    /// Boxed only because a `Resolved` is several hundred bytes -- mostly the
    /// HTTP client inside `YoutubeDl` -- against a playlist variant that is a
    /// string and a pointer.
    Track(Box<Resolved>),
    /// One track, from a link that also named a playlist nobody asked to expand.
    /// The reply says so, since otherwise the option is undiscoverable.
    TrackFromPlaylist(Box<Resolved>),
    /// Never empty: an empty playlist is reported as [`UserError::NoResults`].
    Playlist {
        title: String,
        tracks: Vec<Resolved>,
    },
}

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
    pub async fn resolve(
        &self,
        input: &str,
        requested_by: UserId,
        handling: PlaylistHandling,
    ) -> Result<Resolution, BotError> {
        let attempt = async {
            match classify(input) {
                // Nothing but a list: there is no single track to fall back to,
                // so the flag does not come into it.
                Query::PlaylistOnly(url) => self.resolve_playlist(url, requested_by).await,

                Query::Link { url, in_playlist } => {
                    if in_playlist && handling == PlaylistHandling::Expand {
                        return self.resolve_playlist(url, requested_by).await;
                    }

                    let track = Box::new(self.resolve_link(url, requested_by).await?);

                    Ok(if in_playlist {
                        Resolution::TrackFromPlaylist(track)
                    } else {
                        Resolution::Track(track)
                    })
                }

                Query::Search(terms) => self
                    .resolve_search(terms, requested_by)
                    .await
                    .map(|track| Resolution::Track(Box::new(track))),
            }
        };

        let started = Instant::now();
        let outcome = tokio::time::timeout(RESOLVE_DEADLINE, attempt).await;
        let took = started.elapsed();

        match &outcome {
            Ok(Ok(resolution)) if took >= SLOW_RESOLVE => warn!(
                seconds = took.as_secs_f32(),
                found = resolution.describe(),
                "yt-dlp was slow: YouTube is most likely making it solve a JS \
                 challenge. Check that deno is on PATH and that YTDLP_EXTRA_ARGS \
                 carries --remote-components ejs:github"
            ),
            Ok(Ok(resolution)) => info!(
                seconds = took.as_secs_f32(),
                found = resolution.describe(),
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
        let mut source = self.lazy_source(url.clone());
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

    /// Queue a whole playlist.
    ///
    /// The flat listing already carries a title, duration, uploader and
    /// thumbnail for every entry, so a hundred-track playlist costs exactly one
    /// yt-dlp call. Each track's stream URL is still resolved lazily at playback
    /// time, which is also the only correct moment: those URLs expire.
    async fn resolve_playlist(
        &self,
        url: String,
        requested_by: UserId,
    ) -> Result<Resolution, BotError> {
        let listing = self.flat_list(&url, PLAYLIST_LIMIT).await?;
        let title = listing.title.unwrap_or_else(|| "Playlist".to_owned());

        let tracks: Vec<Resolved> = listing
            .entries
            .into_iter()
            .filter_map(|entry| entry.into_track(requested_by))
            .map(|(song, source_url)| Resolved {
                song,
                source: self.lazy_source(source_url),
            })
            .collect();

        if tracks.is_empty() {
            return Err(UserError::NoResults { query: url }.into());
        }

        Ok(Resolution::Playlist { title, tracks })
    }

    /// The URL of the first search result that is an actual video.
    async fn first_video_for(&self, terms: &str) -> Result<String, BotError> {
        let found = self
            .flat_list(&format!("ytsearch{SEARCH_RESULTS}:{terms}"), SEARCH_RESULTS)
            .await?;

        first_video(found.entries).ok_or_else(|| {
            BotError::from(UserError::NoResults {
                query: terms.to_owned(),
            })
        })
    }

    /// List what `target` contains without extracting any of it.
    ///
    /// `--flat-playlist` is the whole point: extraction is what hangs, both on a
    /// channel returned by a search and on a long playlist.
    async fn flat_list(&self, target: &str, limit: usize) -> Result<FlatListing, BotError> {
        let listing = Command::new(self.program)
            .args(&self.extra_args)
            .args([
                "--flat-playlist",
                "--dump-single-json",
                "--playlist-end",
                &limit.to_string(),
                target,
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

        serde_json::from_slice(&listing.stdout)
            .map_err(|failure| BotError::Search(format!("unreadable listing: {failure}")))
    }

    fn lazy_source(&self, url: String) -> YoutubeDl<'static> {
        YoutubeDl::new_ytdl_like(self.program, self.http.clone(), url)
            .user_args(self.extra_args.clone())
    }
}

/// As much of `--flat-playlist --dump-single-json` as is worth reading.
///
/// A listing entry already carries everything a [`Song`] needs, which is why a
/// playlist costs one yt-dlp call rather than one per track.
#[derive(Deserialize)]
struct FlatListing {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    entries: Vec<FlatEntry>,
}

#[derive(Deserialize, Default)]
struct FlatEntry {
    /// Which extractor yt-dlp would use. See [`VIDEO_EXTRACTOR`].
    #[serde(default)]
    ie_key: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    uploader: Option<String>,
    #[serde(default)]
    thumbnails: Vec<FlatThumbnail>,
}

#[derive(Deserialize, Default)]
struct FlatThumbnail {
    #[serde(default)]
    url: Option<String>,
}

impl FlatEntry {
    /// Become a song, or nothing if this entry is not a playable video.
    ///
    /// Goes through `AuxMetadata` so that a track from a listing and a track
    /// from a full extraction are built by the same code.
    fn into_track(self, requested_by: UserId) -> Option<(Song, String)> {
        if self.ie_key.as_deref() != Some(VIDEO_EXTRACTOR) {
            return None;
        }

        let url = self.url?;

        let metadata = AuxMetadata {
            title: self.title,
            duration: self.duration.map(Duration::from_secs_f64),
            // Listings order thumbnails smallest first.
            thumbnail: self.thumbnails.into_iter().next_back().and_then(|t| t.url),
            source_url: Some(url.clone()),
            channel: self.channel.or(self.uploader),
            ..Default::default()
        };

        Some((Song::from_metadata(metadata, &url, requested_by), url))
    }
}

impl Resolution {
    /// One line naming what was found, for the log.
    fn describe(&self) -> String {
        match self {
            Self::Track(resolved) | Self::TrackFromPlaylist(resolved) => {
                resolved.song.title.clone()
            }
            Self::Playlist { title, tracks } => format!("{title} ({} tracks)", tracks.len()),
        }
    }
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
    /// Names a list and no particular track, so there is nothing to choose.
    PlaylistOnly(String),
    /// Handed to yt-dlp verbatim. That covers every site it supports -- SoundCloud,
    /// Bandcamp, a bare .mp3 -- not just YouTube. `in_playlist` records that the
    /// link also named a list, without deciding what to do about it.
    Link { url: String, in_playlist: bool },
    /// Becomes a YouTube search.
    Search(String),
}

fn classify(input: &str) -> Query {
    let trimmed = input.trim();

    let looks_like_a_link = trimmed
        .split_once("://")
        .is_some_and(|(scheme, rest)| matches!(scheme, "http" | "https") && !rest.is_empty());

    if !looks_like_a_link || trimmed.contains(char::is_whitespace) {
        return Query::Search(trimmed.to_owned());
    }

    let names_a_video =
        trimmed.contains("watch?v=") || trimmed.contains("&v=") || trimmed.contains("youtu.be/");
    let names_a_list = trimmed.contains("list=");

    // Report what the link names; let the caller decide what to do about it.
    match (names_a_list, names_a_video) {
        (true, false) => Query::PlaylistOnly(trimmed.to_owned()),
        (in_playlist, _) => Query::Link {
            url: trimmed.to_owned(),
            in_playlist,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ie_key: &str, url: &str) -> FlatEntry {
        FlatEntry {
            ie_key: Some(ie_key.to_owned()),
            url: Some(url.to_owned()),
            ..Default::default()
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
                ..Default::default()
            },
            entry("Youtube", "https://www.youtube.com/watch?v=good"),
        ];

        assert_eq!(
            first_video(results).as_deref(),
            Some("https://www.youtube.com/watch?v=good")
        );
    }

    fn is_link(input: &str) -> bool {
        matches!(classify(input), Query::Link { .. })
    }

    fn is_playlist_only(input: &str) -> bool {
        matches!(classify(input), Query::PlaylistOnly(_))
    }

    fn sits_in_a_playlist(input: &str) -> bool {
        matches!(
            classify(input),
            Query::Link {
                in_playlist: true,
                ..
            }
        )
    }

    #[test]
    fn a_link_naming_only_a_list_has_no_single_track_to_choose() {
        assert!(is_playlist_only(
            "https://www.youtube.com/playlist?list=UUJhe07czqzg4UzpjvQWkmMQ"
        ));
    }

    /// YouTube's share button gives this shape for a track inside a playlist, so
    /// it has to stay a link -- but one that remembers a list was named, which is
    /// what lets the reply offer to queue the rest.
    #[test]
    fn a_link_naming_both_is_a_track_that_knows_about_its_playlist() {
        for url in [
            "https://www.youtube.com/watch?v=DWaB4PXCwFU&list=PLabc123",
            "https://youtu.be/DWaB4PXCwFU?list=PLabc123",
        ] {
            assert!(is_link(url), "{url} should stay a link");
            assert!(sits_in_a_playlist(url), "{url} should remember the list");
        }
    }

    #[test]
    fn a_plain_video_link_knows_of_no_playlist() {
        assert!(!sits_in_a_playlist(
            "https://www.youtube.com/watch?v=DWaB4PXCwFU"
        ));
    }

    #[test]
    fn words_mentioning_a_list_are_still_a_search() {
        assert!(!is_playlist_only("best playlist ever"));
        assert!(!is_link("best playlist ever"));
    }

    #[test]
    fn expanding_a_playlist_is_opt_in() {
        assert_eq!(
            PlaylistHandling::from_flag(None),
            PlaylistHandling::SingleTrack
        );
        assert_eq!(
            PlaylistHandling::from_flag(Some(false)),
            PlaylistHandling::SingleTrack
        );
        assert_eq!(
            PlaylistHandling::from_flag(Some(true)),
            PlaylistHandling::Expand
        );
    }

    #[test]
    fn a_listing_entry_becomes_a_song() {
        let entry = FlatEntry {
            ie_key: Some("Youtube".to_owned()),
            url: Some("https://www.youtube.com/watch?v=XG5mPjskAZU".to_owned()),
            title: Some("Dear Agony".to_owned()),
            duration: Some(196.0),
            channel: Some("Breaking Benjamin".to_owned()),
            uploader: None,
            thumbnails: vec![
                FlatThumbnail {
                    url: Some("https://i.ytimg.com/small.jpg".to_owned()),
                },
                FlatThumbnail {
                    url: Some("https://i.ytimg.com/large.jpg".to_owned()),
                },
            ],
        };

        let (song, url) = entry.into_track(UserId::new(1)).expect("a playable video");

        assert_eq!(song.title, "Dear Agony");
        assert_eq!(song.length.render(), "3:16");
        assert_eq!(song.uploader.as_deref(), Some("Breaking Benjamin"));
        // Listings order thumbnails smallest first, so the last is the best.
        assert_eq!(
            song.thumbnail.as_deref(),
            Some("https://i.ytimg.com/large.jpg")
        );
        assert_eq!(url, song.url);
    }

    #[test]
    fn a_listing_entry_that_is_not_a_video_becomes_nothing() {
        let channel = FlatEntry {
            ie_key: Some("YoutubeTab".to_owned()),
            url: Some("https://www.youtube.com/channel/abc".to_owned()),
            ..Default::default()
        };

        assert!(channel.into_track(UserId::new(1)).is_none());
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
