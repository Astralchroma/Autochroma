mod commands;
mod id;
mod modules;

use crate::{commands::server_info, commands::uptime, modules::module};
use clap::{Args, Parser};
use log::info;
use poise::builtins::{register_globally, register_in_guild};
use poise::{serenity_prelude as serenity, Command, Framework, FrameworkOptions};
use serenity::{ClientBuilder, GatewayIntents};
use sqlx::{migrate, migrate::MigrateError, PgPool};
use std::{fs, io, path::PathBuf, time::SystemTime};
use thiserror::Error;

pub fn get_global_commands() -> Vec<Command<Data, Error>> {
	vec![module(), server_info(), uptime()]
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
	startup_time: SystemTime,
	database: PgPool,
}

#[derive(Parser)]
#[command(version)]
pub struct Arguments {
	#[command(flatten)]
	discord_token: DiscordToken,

	#[command(flatten)]
	database_uri: DatabaseUri,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct DiscordToken {
	/// Specifies a File to load the Discord Token from
	#[arg(long, alias = "token-file")]
	discord_token_file: Option<PathBuf>,

	/// Specifies a Discord Token
	#[arg(long)]
	discord_token: Option<String>,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct DatabaseUri {
	/// Specifies a File to load the Database URI from
	#[arg(long)]
	database_uri_file: Option<PathBuf>,

	/// Specifies a Database URI
	#[arg(long, alias = "connection-uri")]
	database_uri: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), InitializationError> {
	let arguments = Arguments::parse();

	env_logger::init();

	let discord_token = arguments.discord_token.discord_token_file.map_or(
		arguments
			.discord_token
			.discord_token
			.ok_or(InitializationError::MissingDiscordToken),
		|path| Ok(fs::read_to_string(path)?),
	)?;

	let database_uri = arguments.database_uri.database_uri_file.map_or(
		arguments
			.database_uri
			.database_uri
			.ok_or(InitializationError::MissingDatabaseUri),
		|path| Ok(fs::read_to_string(path)?),
	)?;

	let database = PgPool::connect(&database_uri).await?;
	migrate!().run(&database).await?;

	let commands = get_global_commands();
	// Nom::append_commands(&mut commands);

	let framework = Framework::builder()
		.options(FrameworkOptions {
			commands,
			..Default::default()
		})
		.setup(|context, ready, _framework| {
			Box::pin(async move {
				register_globally(context, &get_global_commands()).await?;

				let data = Data {
					startup_time: SystemTime::now(),
					database,
				};

				for guild in &ready.guilds {
					let commands = modules::get_guild_commands(&data, &guild.id).await?;
					if commands.is_empty() {
						continue;
					}
					register_in_guild(context, &commands, guild.id).await?;
				}

				info!("Ready!");

				Ok(data)
			})
		})
		.build();

	ClientBuilder::new(discord_token, GatewayIntents::GUILDS)
		.framework(framework)
		.await?
		.start()
		.await?;

	Ok(())
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InitializationError {
	#[error("discord_token_file or discord_token should be set")]
	MissingDiscordToken,
	#[error("database_uri_file or database_uri should be set")]
	MissingDatabaseUri,
	Io(#[from] io::Error),
	Database(#[from] sqlx::Error),
	Migration(#[from] MigrateError),
	Discord(#[from] serenity::Error),
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct GenericError(&'static str);
