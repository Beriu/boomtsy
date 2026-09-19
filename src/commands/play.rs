use std::{any::Any, sync::Arc};

use poise::CreateReply;
use songbird::{Call, tracks::Track};
use tokio::sync::Mutex;

use crate::{
    Context,
    error::{BotError, UserError},
    player,
    reply::{self, Placement},
    song::Song,
    youtube::{PlaylistHandling, Resolution, Resolved},
};

/// Play a link or a playlist, or search YouTube for something.
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "A link, a playlist, or words to search YouTube for"] query: String,
    #[description = "Queue the whole playlist, when the link is part of one"] playlist: Option<
        bool,
    >,
) -> Result<(), BotError> {
    // Discord wants an answer within three seconds; yt-dlp routinely takes longer.
    ctx.defer().await?;

    let call = player::join_caller(ctx).await?;
    let resolution = ctx
        .data()
        .resolver
        .resolve(
            &query,
            ctx.author().id,
            PlaylistHandling::from_flag(playlist),
        )
        .await?;

    let announcement = match resolution {
        Resolution::Track(resolved) => {
            let (songs, queue_length) = enqueue_all(&call, vec![*resolved]).await;
            let song = only_track(songs, &query)?;

            reply::enqueued(&song, placement_in(queue_length))
        }

        Resolution::TrackFromPlaylist(resolved) => {
            let (songs, queue_length) = enqueue_all(&call, vec![*resolved]).await;
            let song = only_track(songs, &query)?;

            reply::enqueued_from_playlist(&song, placement_in(queue_length))
        }

        Resolution::Playlist { title, tracks } => {
            let added = tracks.len();
            let (songs, _) = enqueue_all(&call, tracks).await;
            let first = only_track(songs, &query)?;

            reply::playlist_added(&title, added, &first)
        }
    };

    ctx.send(CreateReply::default().embed(announcement)).await?;

    Ok(())
}

/// The queue counts what is playing, so anything past the first is waiting
/// behind it.
fn placement_in(queue_length: usize) -> Placement {
    match queue_length {
        0 | 1 => Placement::StartedPlaying,
        length => Placement::Queued {
            position: length - 1,
        },
    }
}

/// Queue every track under one lock, so a playlist lands as a block rather than
/// interleaved with whatever else is being added.
async fn enqueue_all(call: &Arc<Mutex<Call>>, tracks: Vec<Resolved>) -> (Vec<Arc<Song>>, usize) {
    let mut handler = call.lock().await;
    let mut queued = Vec::with_capacity(tracks.len());

    for resolved in tracks {
        let song = Arc::new(resolved.song);

        // The queue carries the song alongside its audio, so `/queue`,
        // `/nowplaying` and the track announcer can read it back out.
        let carried: Arc<dyn Any + Send + Sync> = song.clone();
        handler
            .enqueue(Track::new_with_data(resolved.source.into(), carried))
            .await;

        queued.push(song);
    }

    let queue_length = handler.queue().len();

    (queued, queue_length)
}

/// The resolver never returns an empty result, but the type cannot say so, so the
/// invariant is checked once here rather than unwrapped.
fn only_track(songs: Vec<Arc<Song>>, query: &str) -> Result<Arc<Song>, BotError> {
    songs.into_iter().next().ok_or_else(|| {
        BotError::from(UserError::NoResults {
            query: query.to_owned(),
        })
    })
}
