//! Failures are split by what they *mean*, not by whether they were anticipated.
//!
//! [`UserError`] is a normal outcome the caller caused and can fix; it is echoed
//! back to them verbatim. Everything else is a genuine fault: logged with detail,
//! reported to the channel as a generic apology.

use poise::serenity_prelude as serenity;
use tracing::{error, warn};

use crate::Data;

#[derive(Debug, thiserror::Error)]
pub enum BotError {
    /// Expected: the caller did something the bot cannot act on.
    #[error(transparent)]
    User(#[from] UserError),

    /// Exceptional: the Discord API rejected us or the gateway broke.
    ///
    /// Boxed because `serenity::Error` on its own is larger than every other
    /// variant put together, and it would otherwise be paid for on the happy path
    /// of every command that returns a `Result`.
    #[error("discord api: {0}")]
    Discord(#[source] Box<serenity::Error>),

    /// Exceptional: we could not enter or hold the voice channel.
    #[error("voice connection: {0}")]
    Join(#[from] songbird::error::JoinError),

    /// Exceptional: yt-dlp failed, or returned something unplayable.
    #[error("resolving audio: {0}")]
    Resolve(#[from] songbird::input::AudioStreamError),

    /// Exceptional: songbird rejected a queue operation.
    #[error("playback control: {0}")]
    Control(#[from] songbird::error::ControlError),
}

impl From<serenity::Error> for BotError {
    fn from(error: serenity::Error) -> Self {
        Self::Discord(Box::new(error))
    }
}

/// Outcomes the caller caused. Each variant's message is shown to them as-is,
/// so it is written as a sentence addressed to a person.
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Join a voice channel first, then try again.")]
    CallerNotInVoice,

    #[error("I'm already playing in {channel}. Join me there.")]
    BotBusyElsewhere { channel: serenity::ChannelId },

    #[error("I'm not playing anything right now.")]
    NotConnected,

    #[error("The queue is empty.")]
    QueueEmpty,

    #[error("Nothing is playing right now.")]
    NothingPlaying,

    #[error("I couldn't find anything for **{query}**.")]
    NoResults { query: String },

    /// `guild_only` on every command makes this unreachable in practice, but the
    /// type system cannot know that, so it is handled rather than unwrapped.
    #[error("That only works inside a server.")]
    NotInGuild,
}

/// Central handler for anything a command returns or panics with.
pub async fn handle(error: poise::FrameworkError<'_, Data, BotError>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            let reply = match &error {
                BotError::User(expected) => {
                    // Not a defect: the caller simply needs telling.
                    expected.to_string()
                }
                fault => {
                    error!(command = %ctx.command().qualified_name, %fault, "command failed");
                    "Something went wrong on my end. Try again in a moment.".to_owned()
                }
            };

            if let Err(send_failure) = ctx.say(reply).await {
                warn!(%send_failure, "could not deliver the error reply");
            }
        }

        other => {
            if let Err(fallback_failure) = poise::builtins::on_error(other).await {
                error!(%fallback_failure, "error handler itself failed");
            }
        }
    }
}
