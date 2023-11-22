use poise::{serenity_prelude::GatewayIntents, Framework, FrameworkOptions};
use serde::Deserialize;
use std::{convert::Infallible, fs::File, io, io::Read};
use thiserror::Error;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {}

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
			commands: vec![],
			..Default::default()
		})
		.token(config.token)
		.intents(GatewayIntents::empty())
		.setup(|_context, _ready, _framework| Box::pin(async move { Ok::<Data, Infallible>(Data {}) }));

	Ok(framework.run().await?)
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum InitializationError {
	Io(#[from] io::Error),
	Parse(#[from] hocon::Error),
	Discord(#[from] poise::serenity_prelude::Error),
}
