//! Configuration is read from the environment exactly once, here, at the boundary.
//!
//! A missing or malformed value fails startup with an error naming the offending
//! key, rather than becoming a `None` that has to be re-checked deeper in.

use std::{
    env::{self, VarError},
    str::FromStr,
    time::Duration,
};

use poise::serenity_prelude::GuildId;

/// Everything the bot needs from its environment, already validated.
#[derive(Debug, Clone)]
pub struct Config {
    /// Bot token from the Discord developer portal.
    pub discord_token: String,
    /// Guild to register slash commands into. Per-guild registration is instant;
    /// global registration takes up to an hour to propagate, so a private bot
    /// should always set this.
    pub guild_id: Option<GuildId>,
    /// Name or path of the `yt-dlp` executable.
    pub ytdlp_path: String,
    /// Extra flags passed to every yt-dlp invocation.
    ///
    /// YouTube's extraction surface shifts constantly -- JS challenges, bot
    /// checks, region gates -- and the fixes are nearly always a yt-dlp flag.
    /// Keeping them in the environment means a broken evening is a restart
    /// rather than a rebuild. Split on whitespace, so no flag may contain a space.
    pub ytdlp_extra_args: Vec<String>,
    /// Leave the voice channel after this long with an empty queue.
    pub idle_timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),

    #[error("environment variable {key} is not valid UTF-8")]
    NotUnicode { key: &'static str },

    #[error("environment variable {key} is {value:?}, which is not a valid {expected}")]
    Invalid {
        key: &'static str,
        value: String,
        expected: &'static str,
    },
}

impl Config {
    /// Read and validate the environment. Called once, during startup.
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            discord_token: required("DISCORD_BOT_TOKEN")?,
            guild_id: parse_optional("DISCORD_GUILD_ID", "guild id")?.map(GuildId::new),
            ytdlp_path: optional("YTDLP_PATH")?.unwrap_or_else(|| "yt-dlp".to_owned()),
            ytdlp_extra_args: optional("YTDLP_EXTRA_ARGS")?
                .map(|raw| raw.split_whitespace().map(str::to_owned).collect())
                .unwrap_or_default(),
            idle_timeout: Duration::from_secs(
                parse_optional("IDLE_TIMEOUT_SECONDS", "whole number of seconds")?
                    .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECONDS),
            ),
        })
    }
}

const DEFAULT_IDLE_TIMEOUT_SECONDS: u64 = 300;

fn required(key: &'static str) -> Result<String, ConfigError> {
    optional(key)?.ok_or(ConfigError::Missing(key))
}

/// An unset variable and one set to whitespace mean the same thing: absent.
fn optional(key: &'static str) -> Result<Option<String>, ConfigError> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(ConfigError::NotUnicode { key }),
    }
}

fn parse_optional<T: FromStr>(
    key: &'static str,
    expected: &'static str,
) -> Result<Option<T>, ConfigError> {
    let Some(raw) = optional(key)? else {
        return Ok(None);
    };

    raw.parse().map(Some).map_err(|_| ConfigError::Invalid {
        key,
        value: raw,
        expected,
    })
}
