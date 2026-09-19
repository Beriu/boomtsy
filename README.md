![Boomtsy](https://github.com/Beriu/boomtzy/blob/master/assets/boomtsy-banner.png?raw=true)

# Boomtsy

A Discord music bot that sits in a voice channel and plays what my friends and I
ask it for. Written in Rust, runs on a Raspberry Pi in my living room, invited to
exactly one server. No web UI, no database, no public listing, no plans for any.

**This is vibecoded.** It was written in a conversation with Claude rather than
designed up front — I said what I wanted, it wrote the Rust, and we fixed things
as they broke. I have read it and I understand what it does, but nobody
architected this on a whiteboard. It is a hobby bot for a handful of people.
Treat the code accordingly if you wander in from a search engine.

What that does *not* mean is untested. Everything below under
[Things that will bite you](#things-that-will-bite-you) is a real failure we hit
by running it, not advice copied off a blog.

## Commands

| Command | What it does |
| --- | --- |
| `/play <query>` | A link, or words to search for. Joins your channel and queues it. |
| `/skip` | Skip what's playing. |
| `/stop` | Stop, clear the queue, leave. |
| `/queue` | What's playing and what's next. |
| `/nowplaying` | Just the current track. |
| `/pause` · `/resume` | Hold and release. |

`/play` takes anything yt-dlp takes, so SoundCloud, Bandcamp and a bare `.mp3`
link work as well as YouTube. Words that aren't a URL become a search.

The bot leaves on its own once the queue has been empty for a while.

## Running it on a Pi

The whole point of this thing is the Pi, so Docker is the intended path. The
image builds the binary, fetches yt-dlp and deno, and runs as a non-root user.

```sh
cp .env.example .env     # fill in the token
docker build -t boomtsy .
docker run -d --name boomtsy --restart unless-stopped --env-file .env boomtsy
```

It listens on no ports — it dials out to Discord and nothing dials in.

Pushing to `master` deploys automatically via the workflow in `.github/`, which
runs on a self-hosted runner on the Pi itself. It builds, replaces the running
container and checks it stayed up.

For development on a normal machine:

```sh
cargo run --release
```

That needs **Rust 1.98+**, **cmake** (the `libopus_sys` crate compiles libopus
from source), **yt-dlp**, and **deno** on `PATH`. No ffmpeg — songbird streams
the URL yt-dlp reports and decodes it in-process with symphonia.

## Configuration

Everything comes from the environment, is validated once at startup, and a bad
value stops the process with the offending key named rather than failing weirdly
an hour later.

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `DISCORD_BOT_TOKEN` | **yes** | — | From the developer portal, Bot tab |
| `DISCORD_GUILD_ID` | no, but set it | global | Commands appear instantly instead of in an hour |
| `YTDLP_PATH` | no | `yt-dlp` | If the binary isn't on `PATH` |
| `YTDLP_EXTRA_ARGS` | effectively yes | none | See the JS challenge note below |
| `IDLE_TIMEOUT_SECONDS` | no | `300` | Leave after this long with an empty queue |
| `RUST_LOG` | no | `boomtsy=info,…` | Log filter |

There is no YouTube credential of any kind. yt-dlp scrapes the public site, so
there's no API key, no quota and no billing account. That is the entire reason
this is free.

### Inviting it

Scopes `bot` + `applications.commands`, permissions *View Channel, Send Messages,
Embed Links, Connect, Speak*:

```
https://discord.com/api/oauth2/authorize
  ?client_id=YOUR_APP_ID
  &scope=bot%20applications.commands
  &permissions=3165184
```

Invite it with only `bot` and it will connect happily and never show a single
command. **No privileged intents are needed** — it uses `GUILDS` and
`GUILD_VOICE_STATES`, never message content, so there's nothing to apply for.

## Things that will bite you

The interesting part of this README. Each of these cost real time.

### Voice needs songbird 0.6 or newer, no exceptions

Discord made its **DAVE** end-to-end encryption protocol mandatory for all voice
connections on **1 March 2026**. songbird 0.6.0 (April 2026) is the first release
that speaks it. Anything older — which is every music bot tutorial written before
then — gives you a bot that logs in fine, registers commands fine, and then
cannot join a voice channel at all.

### symphonia codecs are your job, not songbird's

songbird registers only its own Opus decoder plus the DCA and raw readers, and
defers every container format to whatever symphonia has enabled *in your binary*.
Its own manifest declares symphonia with `default-features = false`; the
`aac`/`isomp4`/`mp3` list you'll find in its `Cargo.toml` is under
`[dev-dependencies]` and never reaches you.

So `Cargo.toml` here declares symphonia directly with `mkv`, `ogg`, `isomp4`,
`aac` and `mp3`. YouTube's audio-only formats are WebM/Opus and M4A/AAC; without
`mkv` in particular the bot joins the channel and then fails on every track.

### Searching an artist by name hangs yt-dlp

Search YouTube for `breaking benjamin` and the top result is the artist's
**channel**, not a video. Ask yt-dlp to extract a channel and it walks the entire
catalogue behind it — minutes of work that ends in a track nobody asked for.
`--no-playlist` does not help, because a channel isn't a playlist. Neither does
`--match-filter`, because the filter runs *after* the extraction that's hanging.

So searching happens in two steps: `--flat-playlist` lists the results without
extracting any of them, the first entry with `ie_key == "Youtube"` is picked, and
only that concrete URL is resolved. Roughly three seconds, versus never.

### YouTube's JS challenge needs deno *and* a flag

yt-dlp has to solve a JavaScript challenge to get a stream URL. Without a real JS
runtime it falls back to a pure-Python interpreter and a single lookup takes over
two minutes instead of four seconds. Both halves are required:

- `deno` on `PATH` — the runtime
- `YTDLP_EXTRA_ARGS=--remote-components ejs:github` — fetches the solver it runs

Neither works alone. The Docker image handles both.

### Keep yt-dlp updated

It's an external binary on purpose. YouTube rotates its player ciphers constantly
and yt-dlp ships fixes within days; a scraper vendored into the build would be
stale within a month. Most "the bot stopped working" evenings are one `yt-dlp -U`
away.

Other flags worth knowing, all via `YTDLP_EXTRA_ARGS`:

| Symptom | Flag |
| --- | --- |
| "Sign in to confirm you're not a bot" | `--cookies-from-browser firefox`, or better, a [PO token provider](https://github.com/Brainicism/bgutil-ytdlp-pot-provider) |
| Hangs on a v6-only host | `--source-address 0.0.0.0` |

A lookup that stalls anyway gets abandoned after 45 seconds and the bot says so,
rather than leaving you watching "thinking…" until Discord expires the token.
Every resolve is timed in the log, and anything over ten seconds warns with the
likely cause.

## On the rules

Discord does not prohibit music bots. Its Developer ToS explicitly permits
self-hosted bot integrations and it has never enforced against servers for
running one. The 2021 shutdowns of Groovy and Rythm came from a Google/YouTube
cease-and-desist, not from Discord, and targeted bots operating at millions of
servers while monetizing.

Restreaming YouTube does contravene *YouTube's* terms regardless of scale. This
is one bot, in one private server, for a handful of friends. Judge that for
yourself before deploying it somewhere that matters.

## Known limits

- **Playlists aren't expanded.** songbird passes `--no-playlist`, so a playlist
  link queues only the video it points at.
- **One voice channel per server.** Ask it to play while it's busy elsewhere and
  it tells you where it is.
- **Nothing survives a restart.** The queue lives in memory. Redeploy mid-song
  and the song is gone. For a bot that restarts roughly never, that's fine.

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
  commands/     one file per slash command, from one compiler-checked list
```

There's a TypeScript version of this bot — with a Vue dashboard — still sitting
on `master`. This branch replaced it wholesale.

## License

ISC — see [LICENSE](LICENSE).
