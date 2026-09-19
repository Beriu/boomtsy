//! Boomtsy -- a self-hosted Discord music bot.
//!
//! One process, one job: sit in a voice channel and play what the server asks for.
//!
//! Requires songbird 0.6 or newer. Discord made its DAVE end-to-end encryption
//! protocol mandatory for voice on 1 March 2026, and 0.6.0 is the first release
//! that speaks it -- anything older simply cannot connect to a voice channel.

mod commands;
mod config;
mod error;
mod events;
mod player;
mod reply;
mod song;
mod youtube;

use std::time::Duration;

use poise::serenity_prelude::{self as serenity, GatewayIntents};
use songbird::serenity::SerenityInit;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::{config::Config, error::BotError, youtube::Resolver};

/// Shared, read-only state handed to every command invocation.
pub struct Data {
    pub resolver: Resolver,
    pub idle_timeout: Duration,
}

pub type Context<'a> = poise::Context<'a, Data, BotError>;

#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),

    /// Boxed for the same reason as `BotError::Discord`: `serenity::Error` is
    /// 136 bytes on its own.
    #[error("could not start the discord client: {0}")]
    Client(#[source] Box<serenity::Error>),
}

impl From<serenity::Error> for StartupError {
    fn from(error: serenity::Error) -> Self {
        Self::Client(Box::new(error))
    }
}

#[tokio::main]
async fn main() {
    // A .env file is a local convenience. Its absence is not a problem: in a
    // container the values arrive as real environment variables.
    let _ = dotenvy::dotenv();
    init_tracing();

    if let Err(failure) = run().await {
        error!("{failure}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), StartupError> {
    let Config {
        discord_token,
        guild_id,
        ytdlp_path,
        ytdlp_extra_args,
        idle_timeout,
    } = Config::from_env()?;

    // songbird wants a `&'static str` for the program name, and this one lives as
    // long as the process anyway. One deliberate leak at startup beats threading
    // a lifetime parameter through every command.
    let ytdlp: &'static str = Box::leak(ytdlp_path.into_boxed_str());

    let data = Data {
        resolver: Resolver::new(reqwest::Client::new(), ytdlp, ytdlp_extra_args),
        idle_timeout,
    };

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            on_error: |failure| Box::pin(error::handle(failure)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                register_commands(ctx, framework, guild_id).await?;
                info!(bot = %ready.user.name, guilds = ready.guilds.len(), "connected");
                Ok(data)
            })
        })
        .build();

    // GUILD_VOICE_STATES is what tells us who is sitting in which channel.
    // Neither it nor GUILDS is privileged, so this bot needs no intent review
    // and no message-content access -- it only ever sees its own slash commands.
    let intents = GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .register_songbird()
        .await?;

    // Leave voice channels and close the gateway cleanly on Ctrl-C or `docker stop`,
    // rather than letting the connection time out on Discord's side.
    let shards = client.shard_manager.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("shutdown requested");
            shards.shutdown_all().await;
        }
    });

    client.start().await?;

    Ok(())
}

async fn register_commands(
    ctx: &serenity::Context,
    framework: &poise::Framework<Data, BotError>,
    guild_id: Option<serenity::GuildId>,
) -> Result<(), BotError> {
    let commands = &framework.options().commands;

    match guild_id {
        // Per-guild registration appears immediately, which is what a bot living
        // in one server wants.
        Some(guild) => {
            info!(%guild, count = commands.len(), "registering commands in guild");
            poise::builtins::register_in_guild(ctx, commands, guild).await?;
        }
        // Global registration can take up to an hour to propagate.
        None => {
            info!(count = commands.len(), "registering commands globally");
            poise::builtins::register_globally(ctx, commands).await?;
        }
    }

    Ok(())
}

const DEFAULT_LOG_FILTER: &str = "boomtsy=info,songbird=warn,serenity=warn";

fn init_tracing() {
    // Same rule the rest of the configuration uses: a variable set to nothing
    // means the same as one that is not set at all. `try_from_default_env`
    // does not agree -- it reads `RUST_LOG=` as a filter with no directives,
    // which silences every log line the process would ever emit. A bare
    // `RUST_LOG=` in a .env file is a very easy way to end up with a bot that
    // looks dead and is in fact running perfectly.
    let filter = std::env::var("RUST_LOG")
        .ok()
        .filter(|directives| !directives.trim().is_empty())
        .and_then(|directives| EnvFilter::try_new(directives).ok())
        .unwrap_or_else(|| EnvFilter::new(DEFAULT_LOG_FILTER));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
