use poise::CreateReply;

use crate::{
    Context,
    error::{BotError, UserError},
    player, reply,
    song::Song,
};

/// Show what is playing and what comes next.
#[poise::command(slash_command, guild_only)]
pub async fn queue(ctx: Context<'_>) -> Result<(), BotError> {
    let call = player::require_call(ctx).await?;

    let (playing, upcoming) = {
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();

        let Some((playing, upcoming)) = tracks.split_first() else {
            return Err(UserError::QueueEmpty.into());
        };

        (
            playing.data::<Song>(),
            upcoming
                .iter()
                .map(|track| track.data::<Song>())
                .collect::<Vec<_>>(),
        )
    };

    ctx.send(CreateReply::default().embed(reply::queue_listing(&playing, &upcoming)))
        .await?;

    Ok(())
}
