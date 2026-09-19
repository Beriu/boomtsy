use crate::{
    Context,
    error::{BotError, UserError},
    player,
    song::Song,
};

/// Skip the track that is playing.
#[poise::command(slash_command, guild_only)]
pub async fn skip(ctx: Context<'_>) -> Result<(), BotError> {
    let call = player::require_call(ctx).await?;

    let (skipped, remaining) = {
        let handler = call.lock().await;
        let queue = handler.queue();

        let Some(playing) = queue.current() else {
            return Err(UserError::NothingPlaying.into());
        };

        let skipped = playing.data::<Song>();
        queue.skip()?;

        (skipped, queue.len().saturating_sub(1))
    };

    let followup = match remaining {
        0 => "Nothing left in the queue.".to_owned(),
        1 => "1 track left.".to_owned(),
        count => format!("{count} tracks left."),
    };

    ctx.say(format!("Skipped **{}**. {followup}", skipped.title))
        .await?;

    Ok(())
}
