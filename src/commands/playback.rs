use crate::{
    Context,
    error::{BotError, UserError},
    player,
};

/// Pause playback.
#[poise::command(slash_command, guild_only)]
pub async fn pause(ctx: Context<'_>) -> Result<(), BotError> {
    let call = player::require_call(ctx).await?;

    {
        let handler = call.lock().await;
        let queue = handler.queue();

        if queue.current().is_none() {
            return Err(UserError::NothingPlaying.into());
        }

        queue.pause()?;
    }

    ctx.say("Paused.").await?;

    Ok(())
}

/// Resume playback.
#[poise::command(slash_command, guild_only)]
pub async fn resume(ctx: Context<'_>) -> Result<(), BotError> {
    let call = player::require_call(ctx).await?;

    {
        let handler = call.lock().await;
        let queue = handler.queue();

        if queue.current().is_none() {
            return Err(UserError::NothingPlaying.into());
        }

        queue.resume()?;
    }

    ctx.say("Back on.").await?;

    Ok(())
}
