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

# The release asset named plain `yt-dlp` is a Python zipapp and needs python3 on
# PATH; only the per-architecture `yt-dlp_linux*` builds are self contained. The
# version check at the end is the point: getting this wrong produces an image
# that builds, starts, connects to Discord, and then fails on every single
# /play, which is a slow and confusing way to find out.
#
# ffmpeg is deliberately absent: songbird streams the URL yt-dlp reports and
# decodes it in-process with symphonia, so nothing ever shells out to ffmpeg.
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends ca-certificates curl; \
    case "$(dpkg --print-architecture)" in \
        amd64) asset=yt-dlp_linux ;; \
        arm64) asset=yt-dlp_linux_aarch64 ;; \
        *) echo "no self-contained yt-dlp build for $(dpkg --print-architecture)" >&2; exit 1 ;; \
    esac; \
    curl -fsSL "https://github.com/yt-dlp/yt-dlp/releases/latest/download/$asset" \
        -o /usr/local/bin/yt-dlp; \
    chmod 0755 /usr/local/bin/yt-dlp; \
    yt-dlp --version; \
    apt-get purge -y curl; \
    apt-get autoremove -y; \
    rm -rf /var/lib/apt/lists/*

# deno is the only JavaScript runtime yt-dlp enables by default, and extraction
# without one is deprecated: it still works, but warns that some formats may be
# missing. Having the binary on PATH is the whole requirement -- no flags needed.
# It is also the largest thing in this image by some margin, which is what rules
# out an Alpine base: deno ships no musl builds.
COPY --from=denoland/deno:bin /deno /usr/local/bin/deno
RUN deno --version

# A real home directory, because yt-dlp caches extractor state under $HOME.
RUN useradd --create-home --uid 10001 boomtsy
USER boomtsy
WORKDIR /home/boomtsy

COPY --from=build /src/target/release/boomtsy /usr/local/bin/boomtsy

# No EXPOSE: the bot dials out to Discord's gateway and never listens.
ENTRYPOINT ["boomtsy"]
