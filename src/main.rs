mod commands;

use poise::{builtins::register_globally, serenity_prelude::GatewayIntents, Framework, FrameworkOptions};
use serde::Deserialize;
use std::{fs::File, io, io::Read, time::SystemTime};
use thiserror::Error;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
	startup_time: SystemTime,
}

#[derive(Deserialize)]
struct Config {
	token: Box<str>,
}

#[tokio::main]
async fn main() -> Result<(), InitializationError> {
	let config: Config = {
		let mut file = File::open("autochroma.conf")?;
		let size = file.metadata()?.len();
		let mut string = String::with_capacity(size as usize);
		file.read_to_string(&mut string)?;
		hocon::de::from_str(&string)?
	};

	let framework = Framework::builder()
		.options(FrameworkOptions {
			commands: vec![commands::uptime()],
			..Default::default()
		})
		.token(config.token)
		.intents(GatewayIntents::empty())
		.setup(|context, _ready, framework| {
			Box::pin(async move {
				register_globally(context, &framework.options().commands).await?;

				Ok(Data {
					startup_time: SystemTime::now(),
				})
			})
		});

	Ok(framework.run().await?)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InitializationError {
	Io(#[from] io::Error),
	Parse(#[from] hocon::Error),
	Discord(#[from] poise::serenity_prelude::Error),
}
