//! Every slash command the bot exposes.
//!
//! The list below is the single registration point, and it is checked by the
//! compiler: a command whose signature drifts from what the framework expects
//! fails the build rather than failing at runtime for whoever types it.

mod nowplaying;
mod play;
mod playback;
mod queue;
mod skip;
mod stop;

use crate::{Data, error::BotError};

pub fn all() -> Vec<poise::Command<Data, BotError>> {
    vec![
        play::play(),
        skip::skip(),
        stop::stop(),
        queue::queue(),
        nowplaying::nowplaying(),
        playback::pause(),
        playback::resume(),
    ]
}
