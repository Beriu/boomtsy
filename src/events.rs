//! Handlers songbird runs on our behalf once the bot is in a channel.

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use poise::serenity_prelude::{self as serenity, ChannelId, CreateMessage, GuildId, async_trait};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{reply, song::Song};

/// How often the idle check runs. Short enough to feel responsive, long enough
/// that it costs nothing.
pub const IDLE_CHECK_INTERVAL: Duration = Duration::from_secs(15);

/// Posts a "now playing" card each time the queue advances, in the channel the
/// session was started from.
pub struct Announcer {
    http: Arc<serenity::Http>,
    channel: ChannelId,
    /// The first track to play after joining is the one `/play` just replied
    /// about, so announcing it would say the same thing twice. Every later start
    /// is a queue advance nobody has seen yet.
    awaiting_first: AtomicBool,
}

impl Announcer {
    pub fn new(http: Arc<serenity::Http>, channel: ChannelId) -> Self {
        Self {
            http,
            channel,
            awaiting_first: AtomicBool::new(true),
        }
    }
}

#[async_trait]
impl VoiceEventHandler for Announcer {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let EventContext::Track(started) = ctx else {
            return None;
        };

        if self.awaiting_first.swap(false, Ordering::Relaxed) {
            return None;
        }

        for (_state, handle) in *started {
            // Every track this bot enqueues carries its `Song`; see `commands::play`.
            let song = handle.data::<Song>();
            let message = CreateMessage::new().embed(reply::now_playing(&song));

            if let Err(error) = self.channel.send_message(&self.http, message).await {
                warn!(%error, "could not announce the next track");
            }
        }

        None
    }
}

/// Leaves the voice channel once the queue has been empty for long enough.
///
/// Without this the bot sits in an empty channel forever, holding a voice
/// connection and showing as present to everyone in the server.
pub struct IdleWatcher {
    manager: Arc<Songbird>,
    guild_id: GuildId,
    timeout: Duration,
    empty_since: Mutex<Option<Instant>>,
}

impl IdleWatcher {
    pub fn new(manager: Arc<Songbird>, guild_id: GuildId, timeout: Duration) -> Self {
        Self {
            manager,
            guild_id,
            timeout,
            empty_since: Mutex::new(None),
        }
    }
}

#[async_trait]
impl VoiceEventHandler for IdleWatcher {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let Some(call) = self.manager.get(self.guild_id) else {
            // Someone already disconnected us; this watcher has nothing left to guard.
            return Some(Event::Cancel);
        };

        let queue_is_empty = {
            let handler = call.lock().await;
            handler.queue().is_empty()
        };

        let mut empty_since = self.empty_since.lock().await;

        if !queue_is_empty {
            *empty_since = None;
            return None;
        }

        let idle_for = empty_since.get_or_insert_with(Instant::now).elapsed();

        if idle_for < self.timeout {
            return None;
        }

        info!(guild = %self.guild_id, "idle timeout reached, leaving voice");

        if let Err(error) = self.manager.remove(self.guild_id).await {
            warn!(%error, "could not leave voice channel after idle timeout");
        }

        Some(Event::Cancel)
    }
}
