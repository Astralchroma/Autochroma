mod commands;
mod modules;

use crate::{commands::server_info, commands::uptime, modules::module, modules::nom::Nom, modules::Module};
use log::info;
use poise::serenity_prelude::{self, GatewayIntents};
use poise::{builtins::register_globally, builtins::register_in_guild, Command, Framework, FrameworkOptions};
use serde::Deserialize;
use sqlx::{migrate, migrate::MigrateError, SqlitePool};
use std::{fs::File, io, io::Read, time::SystemTime};
use thiserror::Error;

pub fn get_global_commands() -> Vec<Command<Data, Error>> {
	vec![module(), server_info(), uptime()]
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Deserialize)]
struct Config {
	token: Box<str>,
}

pub struct Data {
	startup_time: SystemTime,
	database: SqlitePool,
}

#[tokio::main]
async fn main() -> Result<(), InitializationError> {
	env_logger::init();

	let config: Config = {
		let mut file = File::open("autochroma.conf")?;
		let size = file.metadata()?.len();
		let mut string = String::with_capacity(size as usize);
		file.read_to_string(&mut string)?;
		hocon::de::from_str(&string)?
	};

	let database = SqlitePool::connect("sqlite:autochroma.db?mode=rwc").await?;
	migrate!().run(&database).await?;

	let mut commands = get_global_commands();
	Nom::append_commands(&mut commands);

	Ok(Framework::builder()
		.options(FrameworkOptions {
			commands,
			..Default::default()
		})
		.token(config.token)
		.intents(GatewayIntents::GUILDS)
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
		.run()
		.await?)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InitializationError {
	Io(#[from] io::Error),
	Parse(#[from] hocon::Error),
	Database(#[from] sqlx::Error),
	Migration(#[from] MigrateError),
	Discord(#[from] serenity_prelude::Error),
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct GenericError(&'static str);
