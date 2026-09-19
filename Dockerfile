# syntax=docker/dockerfile:1

# ---- build ----------------------------------------------------------------
FROM rust:1.98-bookworm AS build

# libopus is compiled from source by the libopus_sys crate, which drives cmake.
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Compile the dependency tree on its own layer, so that editing a source file
# does not rebuild 460-odd crates including the whole MLS stack behind DAVE.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
 && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src

COPY src ./src
# cargo decides by mtime, and COPY can preserve an older one.
RUN touch src/main.rs && cargo build --release --locked

# ---- runtime --------------------------------------------------------------
FROM debian:bookworm-slim

# yt-dlp ships a self-contained binary; it needs no system Python.
# ffmpeg is deliberately absent: songbird streams the URL yt-dlp reports and
# decodes it in-process with symphonia, so nothing ever shells out to ffmpeg.
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && curl -fsSL https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp \
      -o /usr/local/bin/yt-dlp \
 && chmod 0755 /usr/local/bin/yt-dlp \
 && apt-get purge -y curl \
 && apt-get autoremove -y \
 && rm -rf /var/lib/apt/lists/*

# A real home directory, because yt-dlp caches extractor state under $HOME.
RUN useradd --create-home --uid 10001 boomtsy
USER boomtsy
WORKDIR /home/boomtsy

# deno runs yt-dlp's JS challenge solver. Without it yt-dlp falls back to its
# pure-Python interpreter, where a single YouTube lookup takes upwards of two
# minutes instead of four seconds. It also needs YTDLP_EXTRA_ARGS to carry
# `--remote-components ejs:github`, which is what fetches the solver itself.
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno

COPY --from=build /src/target/release/boomtsy /usr/local/bin/boomtsy

# No EXPOSE: the bot dials out to Discord's gateway and never listens.
ENTRYPOINT ["boomtsy"]
