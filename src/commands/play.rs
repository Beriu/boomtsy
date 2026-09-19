use std::{any::Any, sync::Arc};

use poise::CreateReply;
use songbird::tracks::Track;

use crate::{
    Context,
    error::BotError,
    player,
    reply::{self, Placement},
};

/// Play a link, or search YouTube for something.
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "A link, or words to search YouTube for"] query: String,
) -> Result<(), BotError> {
    // Discord wants an answer within three seconds; yt-dlp routinely takes longer.
    ctx.defer().await?;

    let call = player::join_caller(ctx).await?;
    let resolved = ctx.data().resolver.resolve(&query, ctx.author().id).await?;

    let song = Arc::new(resolved.song);

    // The queue carries the song alongside its audio, so `/queue`, `/nowplaying`
    // and the track announcer can all read it back out of the handle.
    let carried: Arc<dyn Any + Send + Sync> = song.clone();
    let track = Track::new_with_data(resolved.source.into(), carried);

    let queue_length = {
        let mut handler = call.lock().await;
        handler.enqueue(track).await;
        handler.queue().len()
    };

    // The queue counts the track that is playing, so anything past the first is
    // waiting behind it.
    let placement = match queue_length {
        0 | 1 => Placement::StartedPlaying,
        length => Placement::Queued {
            position: length - 1,
        },
    };

    ctx.send(CreateReply::default().embed(reply::enqueued(&song, placement)))
        .await?;

    Ok(())
}
