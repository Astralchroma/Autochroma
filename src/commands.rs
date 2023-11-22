use crate::{Context, Result};
use poise::command;
use std::time::SystemTime;

#[command(slash_command)]
pub async fn uptime(context: Context<'_>) -> Result<()> {
	let time = context
		.data()
		.startup_time
		.duration_since(SystemTime::UNIX_EPOCH)?
		.as_secs();
	let message = format!("Autochroma was started <t:{time}:R> on <t:{time}:D> at <t:{time}:T>.",);
	context.reply(message).await?;
	Ok(())
}
