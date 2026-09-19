//! Getting hold of -- and getting into -- a guild's voice call.

use std::sync::Arc;

use poise::serenity_prelude::{self as serenity, ChannelId};
use songbird::{Call, Event, Songbird, TrackEvent};
use tokio::sync::Mutex;

use crate::{
    Context,
    error::{BotError, UserError},
    events::{Announcer, IDLE_CHECK_INTERVAL, IdleWatcher},
};

/// The voice manager, installed during client construction.
///
/// Its absence would mean the bot was built wrong, not that anything went wrong
/// at runtime -- so this is an assertion, not a recoverable error.
pub async fn manager(ctx: Context<'_>) -> Arc<Songbird> {
    songbird::get(ctx.serenity_context())
        .await
        .expect("songbird is registered in main::run")
}

/// Join the caller's voice channel, or hand back the call we are already holding
/// there. Refuses to abandon a channel that is already being listened to.
pub async fn join_caller(ctx: Context<'_>) -> Result<Arc<Mutex<Call>>, BotError> {
    let guild_id = ctx.guild_id().ok_or(UserError::NotInGuild)?;
    let destination = caller_voice_channel(ctx)?;
    let manager = manager(ctx).await;

    if let Some(existing) = manager.get(guild_id) {
        let occupied = existing.lock().await.current_channel();

        match occupied {
            Some(channel) if channel.0.get() == destination.get() => return Ok(existing),
            Some(channel) => {
                return Err(UserError::BotBusyElsewhere {
                    channel: ChannelId::new(channel.0.get()),
                }
                .into());
            }
            // A call object with no channel is a leftover from a dropped
            // connection: fall through and reconnect it.
            None => {}
        }
    }

    let call = manager.join(guild_id, destination).await?;
    attach_handlers(&call, ctx, manager.clone(), guild_id).await;

    Ok(call)
}

/// The call for this guild, or an error saying there is nothing to act on.
pub async fn require_call(ctx: Context<'_>) -> Result<Arc<Mutex<Call>>, BotError> {
    let guild_id = ctx.guild_id().ok_or(UserError::NotInGuild)?;

    manager(ctx)
        .await
        .get(guild_id)
        .ok_or_else(|| BotError::from(UserError::NotConnected))
}

async fn attach_handlers(
    call: &Arc<Mutex<Call>>,
    ctx: Context<'_>,
    manager: Arc<Songbird>,
    guild_id: serenity::GuildId,
) {
    let mut handler = call.lock().await;

    // Reconnecting into a stale call would otherwise stack a second copy of each.
    handler.remove_all_global_events();

    handler.add_global_event(
        Event::Track(TrackEvent::Play),
        Announcer::new(Arc::clone(&ctx.serenity_context().http), ctx.channel_id()),
    );

    handler.add_global_event(
        Event::Periodic(IDLE_CHECK_INTERVAL, None),
        IdleWatcher::new(manager, guild_id, ctx.data().idle_timeout),
    );
}

/// Which channel the person who typed the command is sitting in.
///
/// Reads the cache, so it needs no API round trip -- but the cache guard is not
/// `Send`, which is why this is a synchronous function that returns a plain id.
fn caller_voice_channel(ctx: Context<'_>) -> Result<ChannelId, BotError> {
    let guild = ctx.guild().ok_or(UserError::NotInGuild)?;

    guild
        .voice_states
        .get(&ctx.author().id)
        .and_then(|state| state.channel_id)
        .ok_or_else(|| BotError::from(UserError::CallerNotInVoice))
}
