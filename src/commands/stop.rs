use crate::{
    Context,
    error::{BotError, UserError},
    player,
};

/// Stop playing, clear the queue and leave the channel.
#[poise::command(slash_command, guild_only)]
pub async fn stop(ctx: Context<'_>) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(UserError::NotInGuild)?;
    let call = player::require_call(ctx).await?;

    {
        let handler = call.lock().await;
        handler.queue().stop();
    }

    player::manager(ctx).await.remove(guild_id).await?;

    ctx.say("Stopped, queue cleared. See you.").await?;

    Ok(())
}
