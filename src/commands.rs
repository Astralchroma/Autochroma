use crate::{modules::get_formatted_module_list, Context, Result};
use poise::command;
use std::time::SystemTime;

#[command(guild_only, slash_command)]
pub async fn server_info(context: Context<'_>) -> Result<()> {
	let guild = context.guild().expect("command is guild only");
	let module_list = get_formatted_module_list(&context).await?;

	context
		.send(|reply| {
			reply.embed(|embed| {
				embed
					.title(&guild.name)
					.thumbnail(guild.icon_url().unwrap_or(String::from("")))
					.image(guild.banner_url().unwrap_or(String::from("")))
					.description(guild.description.unwrap_or(String::from("")))
					.field("Owner", format!("<@{}>", guild.owner_id), true)
					.field("Members", guild.member_count, true)
					.field("Channels", guild.channels.len(), true)
					.field("Enabled Modules", module_list, false)
			})
		})
		.await?;

	Ok(())
}

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
