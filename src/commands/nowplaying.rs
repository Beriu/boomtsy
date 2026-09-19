use poise::CreateReply;

use crate::{
    Context,
    error::{BotError, UserError},
    player, reply,
    song::Song,
};

/// Show the track that is playing.
#[poise::command(slash_command, guild_only, rename = "nowplaying")]
pub async fn nowplaying(ctx: Context<'_>) -> Result<(), BotError> {
    let call = player::require_call(ctx).await?;

    let playing = {
        let handler = call.lock().await;

        let Some(playing) = handler.queue().current() else {
            return Err(UserError::NothingPlaying.into());
        };

        playing.data::<Song>()
    };

    ctx.send(CreateReply::default().embed(reply::now_playing(&playing)))
        .await?;

    Ok(())
}
