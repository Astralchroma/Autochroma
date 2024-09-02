use crate::{id::Id, modules::get_formatted_module_list, Context, Result};
use poise::{command, serenity_prelude::CreateEmbed, CreateReply};
use std::time::SystemTime;

#[command(slash_command, guild_only)]
pub async fn server_info(context: Context<'_>) -> Result<()> {
	let id = context.guild_id().expect("command is guild only");
	let module_list = get_formatted_module_list(Id::from(id), &context.data().database).await?;

	let reply = {
		let guild = context.guild().expect("command is guild only");
		CreateReply::default().embed(
			CreateEmbed::default()
				.title(&guild.name)
				.thumbnail(guild.icon_url().unwrap_or(String::from("")))
				.image(guild.banner_url().unwrap_or(String::from("")))
				.description(guild.description.clone().unwrap_or(String::from("")))
				.field("Owner", format!("<@{}>", guild.owner_id), true)
				.field("Members", guild.member_count.to_string(), true)
				.field("Channels", guild.channels.len().to_string(), true)
				.field("Enabled Modules", module_list, false),
		)
	};

	context.send(reply).await?;

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
