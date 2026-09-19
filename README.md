![Boomtsy](https://github.com/Beriu/boomtzy/blob/master/assets/boomtsy-banner.png?raw=true)

# Boomtsy

Self-hosted Discord music bot in Rust. Streams YouTube — and anything else
yt-dlp supports — into a voice channel. Built to run on a Raspberry Pi.

## Commands

| Command | |
| --- | --- |
| `/play <query> [playlist]` | Link, playlist, or search terms. Joins your channel and queues it. |
| `/skip` | Skip the current track. |
| `/stop` | Stop, clear the queue, leave. |
| `/queue` | Current track and what's next. |
| `/nowplaying` | Current track. |
| `/pause` · `/resume` | |

Leaves automatically once the queue has been empty for `IDLE_TIMEOUT_SECONDS`.

### Playlists

YouTube's share button gives `watch?v=…&list=…` for a track inside a playlist,
which can mean either the track or the list. `/play` queues **just the track** and
says the link was part of a playlist; add `playlist:True` to queue the whole
thing (capped at 100). A link naming only a list — `playlist?list=…` — always
queues everything, since there is no single track to choose.

## Requirements

- Rust 1.98+
- `cmake` — `libopus_sys` builds libopus from source
- `yt-dlp` and `deno` on `PATH`
- No ffmpeg — songbird decodes in-process via symphonia

## Configuration

Read from the environment once at startup; a bad value exits naming the key.

| Variable | Required | Default |
| --- | --- | --- |
| `DISCORD_BOT_TOKEN` | yes | — |
| `DISCORD_GUILD_ID` | recommended | global registration |
| `YTDLP_PATH` | no | `yt-dlp` |
| `YTDLP_EXTRA_ARGS` | see below | none |
| `IDLE_TIMEOUT_SECONDS` | no | `300` |
| `RUST_LOG` | no | `boomtsy=info,songbird=warn,serenity=warn` |

No YouTube credentials — yt-dlp scrapes the public site, so there's no API key
or quota.

## Running

```sh
cp .env.example .env
docker build -t boomtsy .
docker run -d --name boomtsy --restart unless-stopped --env-file .env boomtsy
```

No ports are exposed; the bot only dials out.

Locally: `cargo run --release`.

## Invite

Scopes `bot` + `applications.commands`, permissions `3165184` (View Channel,
Send Messages, Embed Links, Connect, Speak):

```
https://discord.com/api/oauth2/authorize?client_id=YOUR_APP_ID&scope=bot%20applications.commands&permissions=3165184
```

Without `applications.commands` the bot connects but no commands appear. No
privileged intents required — it uses `GUILDS` and `GUILD_VOICE_STATES` only.

## Gotchas

**songbird 0.6+ is mandatory.** Discord's DAVE voice encryption became required
on 1 March 2026; 0.6.0 is the first release implementing it. Older versions
connect and register commands, then fail to join voice.

**symphonia codec features are the application's job.** songbird registers only
Opus, DCA and raw, and declares symphonia with `default-features = false`.
`Cargo.toml` here enables `mkv`, `ogg`, `isomp4`, `aac`, `mp3` — YouTube serves
WebM/Opus and M4A/AAC, so without `mkv` every track fails.

**Artist-name searches return a channel, not a video.** Extracting a channel
makes yt-dlp walk its whole catalogue. `--no-playlist` doesn't apply and
`--match-filter` runs after extraction. Search is therefore two steps:
`--flat-playlist` to list, pick the first `ie_key == "Youtube"` entry, then
resolve that URL.

**yt-dlp needs a JavaScript runtime.** Without one it warns that *"YouTube
extraction without a JS runtime has been deprecated, and some formats may be
missing"*. It currently still works, but it is on the way out. deno is the only
runtime yt-dlp enables by default, and having it on `PATH` is enough — no flags,
no `--remote-components`. The Docker image includes it.

**Keep yt-dlp updated** (`yt-dlp -U`). It's an external binary because YouTube
changes constantly. Other useful flags: `--cookies-from-browser firefox` for bot
checks, `--source-address 0.0.0.0` on v6-only hosts.

Resolves are timed in the log and abandoned after 45s with a message to the user.

## Limits

- Playlists are capped at 100 tracks
- One voice channel per guild
- Queue is in memory and does not survive a restart

## License

[MIT](LICENSE) — provided as is, without warranty or liability.
