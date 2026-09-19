//! Every embed the bot sends. Kept apart from the commands so that wording and
//! layout stay consistent, and so each builder is a plain total function.

use std::sync::Arc;

use poise::serenity_prelude::{Colour, CreateEmbed, CreateEmbedFooter};

use crate::song::Song;

/// Discord blurple, so the bot reads as part of the client rather than against it.
const ACCENT: Colour = Colour::new(0x5865F2);

/// Where a freshly added track landed. Replaces the "is it playing? what position?"
/// pair of flags that this would otherwise be.
pub enum Placement {
    StartedPlaying,
    Queued { position: usize },
}

/// Confirmation for a track the user just added.
pub fn enqueued(song: &Song, placement: Placement) -> CreateEmbed {
    let heading = match placement {
        Placement::StartedPlaying => "Now playing".to_owned(),
        Placement::Queued { position } => format!("Queued - #{position}"),
    };

    decorate(base(song).author(author_line(heading)), song)
}

/// Announcement posted when the queue advances on its own.
pub fn now_playing(song: &Song) -> CreateEmbed {
    decorate(
        base(song).author(author_line("Now playing".to_owned())),
        song,
    )
}

/// The current track plus what follows it.
pub fn queue_listing(current: &Song, upcoming: &[Arc<Song>]) -> CreateEmbed {
    let lines = upcoming
        .iter()
        .enumerate()
        .take(MAX_LISTED)
        .map(|(index, song)| {
            format!(
                "`{}.` [{}]({}) `{}`",
                index + 1,
                escape_brackets(&song.title),
                song.url,
                song.length.render()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let body = if lines.is_empty() {
        "*Nothing queued up next.*".to_owned()
    } else {
        lines
    };

    let overflow = upcoming.len().saturating_sub(MAX_LISTED);
    let footer = if overflow > 0 {
        format!("{} more not shown", overflow)
    } else {
        format!("{} in queue", upcoming.len())
    };

    CreateEmbed::new()
        .colour(ACCENT)
        .author(author_line("Queue".to_owned()))
        .title(&current.title)
        .url(&current.url)
        .description(format!(
            "**Playing now** `{}`\n\n{body}",
            current.length.render()
        ))
        .footer(CreateEmbedFooter::new(footer))
}

const MAX_LISTED: usize = 15;

fn base(song: &Song) -> CreateEmbed {
    CreateEmbed::new()
        .colour(ACCENT)
        .title(&song.title)
        .url(&song.url)
}

/// Resolve the optional fields once, here, so no caller has to branch on them.
fn decorate(embed: CreateEmbed, song: &Song) -> CreateEmbed {
    let embed = embed.footer(CreateEmbedFooter::new(match &song.uploader {
        Some(uploader) => format!("{}  -  {}", uploader, song.length.render()),
        None => song.length.render(),
    }));

    match &song.thumbnail {
        Some(url) => embed.thumbnail(url),
        None => embed,
    }
}

fn author_line(text: String) -> poise::serenity_prelude::CreateEmbedAuthor {
    poise::serenity_prelude::CreateEmbedAuthor::new(text)
}

/// Titles routinely contain `[` and `]`, which would break the markdown links
/// they get embedded in.
fn escape_brackets(title: &str) -> String {
    title.replace('[', "(").replace(']', ")")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutralises_brackets_that_would_break_a_markdown_link() {
        assert_eq!(
            escape_brackets("Artist - Title [Official Video]"),
            "Artist - Title (Official Video)"
        );
    }

    #[test]
    fn leaves_ordinary_titles_alone() {
        assert_eq!(escape_brackets("Around The World"), "Around The World");
    }
}
