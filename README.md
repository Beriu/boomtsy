![Boomtsy](https://github.com/Beriu/boomtzy/blob/master/assets/boomtsy-banner.png?raw=true)

# Boomtsy

A self-hosted Discord music bot, in Rust. One process: it sits in a voice
channel and plays what the server asks for. No web UI, no database, no public
listing — it is meant to run in one server, for people who know each other.

## Commands

| Command | What it does |
| --- | --- |
| `/play <query>` | A link, or words to search YouTube for. Joins your channel and queues the track. |
| `/skip` | Skip what's playing. |
| `/stop` | Stop, clear the queue, leave. |
| `/queue` | What's playing and what's next. |
| `/nowplaying` | Just the current track. |
| `/pause` · `/resume` | Hold and release. |

`/play` accepts anything yt-dlp accepts, so SoundCloud, Bandcamp and a bare
`.mp3` link work as well as YouTube. Words that aren't a URL become a search.

The bot leaves on its own once the queue has been empty for `IDLE_TIMEOUT_SECONDS`.

## Requirements

- **Rust 1.98+**
- **cmake** — the `libopus_sys` crate compiles libopus from source
- **yt-dlp** on `PATH` — the one moving part; see [Keeping YouTube working](#keeping-youtube-working)
- **No ffmpeg.** songbird streams the URL yt-dlp reports and decodes it in-process
  with symphonia.

## Setting up the Discord app

1. Create an app at [discord.com/developers](https://discord.com/developers/applications),
   add a **Bot**, and copy its token into `DISCORD_BOT_TOKEN`.
2. **No privileged intents required.** The bot uses `GUILDS` and
   `GUILD_VOICE_STATES` only — it never reads message content, so there is
   nothing to apply for and nothing to review.
3. Invite it with scopes `bot` + `applications.commands` and permissions
   *View Channel, Send Messages, Embed Links, Connect, Speak*:

   ```
   https://discord.com/api/oauth2/authorize
     ?client_id=YOUR_APP_ID
     &scope=bot%20applications.commands
     &permissions=3165184
   ```

4. Set `DISCORD_GUILD_ID` to your server. Commands registered per-guild appear
   the instant the bot connects; registered globally they take up to an hour.

## Configuration

Everything comes from the environment, is validated once at startup, and a bad
value stops the process with the offending key named. Copy `.env.example` to
`.env` for local runs.

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `DISCORD_BOT_TOKEN` | yes | — | Bot token |
| `DISCORD_GUILD_ID` | no | global | Register commands into one guild |
| `YTDLP_PATH` | no | `yt-dlp` | Where the binary lives |
| `YTDLP_EXTRA_ARGS` | no | none | Extra yt-dlp flags, whitespace-split |
| `IDLE_TIMEOUT_SECONDS` | no | `300` | Leave after this long with an empty queue |
| `RUST_LOG` | no | `boomtsy=info,...` | Log filter |

## Running it

```sh
cp .env.example .env    # fill in the token
cargo run --release
```

Or in Docker — the image builds the binary, fetches yt-dlp, and runs as a
non-root user:

```sh
docker build -t boomtsy .
docker run -d --name boomtsy --restart unless-stopped --env-file .env boomtsy
```

The container listens on no ports; it only dials out to Discord.

## Keeping YouTube working

yt-dlp is an external binary on purpose. YouTube rotates its player ciphers and
bot checks constantly, and yt-dlp ships fixes within days — a scraper vendored
into the build would be stale within a month. **Update it regularly**; most
"the bot stopped working" evenings are one `yt-dlp -U` away.

When YouTube changes something that needs a flag rather than an update, put it
in `YTDLP_EXTRA_ARGS` and restart — no rebuild:

| Symptom | Flag |
| --- | --- |
| `challenge solver script ... were skipped` | `--remote-components ejs:github` (needs deno) |
| "Sign in to confirm you're not a bot" | `--cookies-from-browser firefox` |
| Hangs on a v6-only host | `--source-address 0.0.0.0` |

## Notes

**Voice requires songbird 0.6+.** Discord made its DAVE end-to-end encryption
protocol mandatory for all voice connections on **1 March 2026**. songbird 0.6.0
is the first release that speaks it. Any older version — and so every music-bot
tutorial written before April 2026 — produces a bot that cannot join a voice
channel at all.

**On the rules.** Discord does not prohibit music bots: its Developer ToS
explicitly permits self-hosted bot integrations, and Discord has never enforced
against servers for running one. The 2021 shutdowns of Groovy and Rythm came
from a Google/YouTube cease-and-desist, not from Discord, and were aimed at bots
operating at millions-of-servers scale. Restreaming YouTube does contravene
*YouTube's* terms regardless of size. This bot is built for one private server;
judge that for yourself.

**Playlists are not expanded.** songbird passes `--no-playlist` to yt-dlp, so a
playlist link queues only the one video it points at.

## Layout

```
src/
  main.rs       boot: config -> framework -> client, and graceful shutdown
  config.rs     the only place the environment is read
  error.rs      expected (tell the user) vs exceptional (log it) failures
  song.rs       the one track description everything else works with
  youtube.rs    query -> playable source, via yt-dlp
  player.rs     joining and holding a guild's voice call
  events.rs     track announcements and the idle-disconnect watcher
  reply.rs      every embed the bot sends
  commands/     one file per slash command, registered from one compiler-checked list
```

The TypeScript version this replaces is still on `master`.

## License

ISC — see [LICENSE](LICENSE).
