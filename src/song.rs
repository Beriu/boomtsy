//! The one description of a track that the rest of the bot works with.
//!
//! yt-dlp hands back a bag of optional fields; every one of them is resolved
//! here, at construction, so that nothing downstream has to ask twice.

use std::time::Duration;

use poise::serenity_prelude::UserId;
use songbird::input::AuxMetadata;

/// How long a track runs.
///
/// yt-dlp reports no duration for a live stream, which is the overwhelmingly
/// common reason for a missing value -- so the absence is modelled as the thing
/// it means rather than left as an `Option` for every caller to interpret.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    Fixed(Duration),
    Live,
}

impl Length {
    /// `7:04`, `1:02:59`, or `LIVE`. Total: every `Length` has a rendering.
    pub fn render(self) -> String {
        let Self::Fixed(duration) = self else {
            return "LIVE".to_owned();
        };

        let total = duration.as_secs();
        let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);

        if hours > 0 {
            format!("{hours}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes}:{seconds:02}")
        }
    }
}

/// A resolved, playable track, carried alongside its audio in songbird's queue.
#[derive(Debug, Clone)]
pub struct Song {
    pub title: String,
    pub url: String,
    pub length: Length,
    pub thumbnail: Option<String>,
    pub uploader: Option<String>,
    pub requested_by: UserId,
}

impl Song {
    /// `fallback_url` is what the user typed: used only when yt-dlp declines to
    /// report a canonical URL, which happens for some direct media links.
    pub fn from_metadata(metadata: AuxMetadata, fallback_url: &str, requested_by: UserId) -> Self {
        Self {
            title: metadata
                .title
                .or(metadata.track)
                .unwrap_or_else(|| "Unknown track".to_owned()),
            url: metadata
                .source_url
                .unwrap_or_else(|| fallback_url.to_owned()),
            length: metadata.duration.map_or(Length::Live, Length::Fixed),
            thumbnail: metadata.thumbnail,
            uploader: metadata.channel.or(metadata.artist),
            requested_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(seconds: u64) -> Length {
        Length::Fixed(Duration::from_secs(seconds))
    }

    #[test]
    fn pads_seconds_but_not_minutes() {
        assert_eq!(fixed(64).render(), "1:04");
        assert_eq!(fixed(242).render(), "4:02");
    }

    #[test]
    fn grows_an_hours_field_only_when_needed() {
        assert_eq!(fixed(3599).render(), "59:59");
        assert_eq!(fixed(3600).render(), "1:00:00");
        assert_eq!(fixed(3661).render(), "1:01:01");
    }

    #[test]
    fn renders_zero_and_live() {
        assert_eq!(fixed(0).render(), "0:00");
        assert_eq!(Length::Live.render(), "LIVE");
    }

    #[test]
    fn a_missing_duration_reads_as_live() {
        let metadata = AuxMetadata {
            title: Some("Some Stream".to_owned()),
            duration: None,
            ..Default::default()
        };

        let song = Song::from_metadata(metadata, "https://example.invalid/s", UserId::new(1));

        assert_eq!(song.length, Length::Live);
    }

    #[test]
    fn falls_back_to_the_typed_url_when_yt_dlp_reports_none() {
        let metadata = AuxMetadata {
            title: None,
            track: None,
            source_url: None,
            ..Default::default()
        };

        let song = Song::from_metadata(metadata, "https://example.invalid/x", UserId::new(1));

        assert_eq!(song.url, "https://example.invalid/x");
        assert_eq!(song.title, "Unknown track");
    }
}
