pub mod nom;

use crate::{modules::nom::Nom, Context, Data, Error, Result};
use log::warn;
use poise::{builtins::register_in_guild, command, serenity_prelude::GuildId, ChoiceParameter, Command};
use sqlx::{query, sqlite::SqliteQueryResult};

#[derive(Debug, ChoiceParameter)]
#[allow(non_camel_case_types)] // I can't be bothered to spam #[name = ?]
pub enum Modules {
	nom,
}

pub trait Module {
	const NAME: &'static str;

	fn append_commands(commands: &mut Vec<Command<Data, Error>>);
}

pub async fn get_guild_commands(data: &Data, guild: &GuildId) -> Result<Vec<Command<Data, Error>>> {
	let guild_id = guild.0 as i64;

	let modules = query!("SELECT id, module FROM modules WHERE guild = ?", guild_id)
		.fetch_all(&data.database)
		.await?;

	let enabled_modules = Vec::with_capacity(modules.len());

	async fn delete_module_by_id(data: &Data, id: i64) -> Result<SqliteQueryResult, sqlx::Error> {
		query!("DELETE FROM modules WHERE id = ?", id)
			.execute(&data.database)
			.await
	}

	let mut commands = vec![];

	for module in modules {
		if enabled_modules.contains(&module.module) {
			warn!("Duplicate module `{}` for {guild}! Removing duplicate.", module.module);
			delete_module_by_id(data, module.id).await?;
			continue;
		}

		match &*module.module {
			"nom" => Nom::append_commands(&mut commands),
			_ => {
				warn!("Invalid module `{}` for {guild}! Removing.", module.module);
				delete_module_by_id(data, module.id).await?;
				continue;
			}
		};
	}

	Ok(commands)
}

/// # Panics
/// If used outside of a guild.
pub async fn get_formatted_module_list(context: &Context<'_>) -> Result<String> {
	let guild_id = context.guild_id().expect("command is guild only").0 as i64;

	let module_list = query!("SELECT module FROM modules WHERE guild = ?", guild_id)
		.fetch_all(&context.data().database)
		.await?
		.into_iter()
		.map(|record| record.module)
		.collect::<Vec<String>>();

	Ok(match module_list.len() {
		0 => String::from("*None*"),
		_ => module_list.join(", "),
	})
}

#[command(owners_only, guild_only, slash_command, subcommands("enable", "list", "disable"))]
pub async fn module(_: Context<'_>) -> Result<()> {
	Ok(())
}

#[command(owners_only, guild_only, slash_command)]
async fn enable(context: Context<'_>, module: Modules) -> Result<()> {
	let guild = context.guild_id().expect("command is guild only");
	let guild_id = guild.0 as i64;
	let name = module.name();
	query!("INSERT INTO modules (guild, module) VALUES (?, ?)", guild_id, name)
		.execute(&context.data().database)
		.await?;
	let commands = get_guild_commands(context.data(), &guild).await?;
	register_in_guild(context, &commands, guild).await?;
	context.reply(format!("Enabled Module: {}", name)).await?;
	Ok(())
}

#[command(owners_only, guild_only, slash_command)]
async fn list(context: Context<'_>) -> Result<()> {
	let module_list = get_formatted_module_list(&context).await?;
	context.reply(format!("Enabled Modules: {}.", module_list)).await?;
	Ok(())
}

#[command(owners_only, guild_only, slash_command)]
async fn disable(context: Context<'_>, module: Modules) -> Result<()> {
	let guild = context.guild_id().expect("command is guild only");
	let guild_id = guild.0 as i64;
	let name = module.name();
	query!("DELETE FROM modules WHERE guild = ? AND module = ?", guild_id, name)
		.execute(&context.data().database)
		.await?;
	let commands = get_guild_commands(context.data(), &guild).await?;
	register_in_guild(context, &commands, guild).await?;
	context.reply(format!("Disabled Module: {}", name)).await?;
	Ok(())
}
