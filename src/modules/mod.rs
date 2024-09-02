use crate::{id::Id, Context, Data, Error, Result};
use log::warn;
use poise::{builtins::register_in_guild, command, serenity_prelude::GuildId, ChoiceParameter, Command};
use sqlx::{postgres::PgQueryResult, prelude::Type, query, query_scalar, Executor, Postgres, Transaction};
use std::fmt::{self, Display, Formatter};

#[derive(ChoiceParameter, Debug, Type, PartialEq)]
#[sqlx(type_name = "module", rename_all = "lowercase")]
pub enum Module {
	#[name = "nom"]
	Nom,
}

impl Display for Module {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
		formatter.write_str(self.name())
	}
}

pub trait ModuleImpl {
	const NAME: &'static str;

	fn append_commands(commands: &mut Vec<Command<Data, Error>>);
}

pub async fn get_guild_commands(data: &Data, guild: &GuildId) -> Result<Vec<Command<Data, Error>>> {
	let id = Id::from(*guild);

	let mut transaction = data.database.begin().await?;

	let modules: Vec<Module> = query_scalar!("SELECT module as \"module: _\" FROM modules WHERE guild = $1", id as _)
		.fetch_all(&mut *transaction)
		.await?;

	let enabled_modules = Vec::with_capacity(modules.len());

	async fn delete_module_by_id(
		transaction: &mut Transaction<'_, Postgres>,
		id: Id,
	) -> Result<PgQueryResult, sqlx::Error> {
		query!("DELETE FROM modules WHERE guild = $1", id as _)
			.execute(&mut **transaction)
			.await
	}

	let commands = vec![];

	for module in modules {
		if enabled_modules.contains(&module) {
			warn!("Duplicate module `{module:?}` for {guild:?}! Removing duplicate.");
			delete_module_by_id(&mut transaction, id).await?;
			continue;
		}

		match &*module.to_string() {
			"nom" => {} // Nom::append_commands(&mut commands),
			_ => {
				warn!("Invalid module `{module:?}` for {guild:?}! Removing.");
				delete_module_by_id(&mut transaction, id).await?;
				continue;
			}
		};
	}

	transaction.commit().await?;

	Ok(commands)
}

/// # Panics
/// If used outside of a guild.
pub async fn get_formatted_module_list<'a>(id: Id, executor: impl Executor<'_, Database = Postgres>) -> Result<String> {
	let module_list = query_scalar!("SELECT module as \"module: _\" FROM modules WHERE guild = $1", id as _)
		.fetch_all(executor)
		.await?
		.into_iter()
		.map(|module: Module| module.to_string())
		.collect::<Vec<_>>();

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
async fn enable(context: Context<'_>, module: Module) -> Result<()> {
	let guild = context.guild_id().expect("command is guild only");
	let id = Id::from(guild);

	query!(
		"INSERT INTO modules (guild, module) VALUES ($1, $2)",
		id as _,
		module as _,
	)
	.execute(&context.data().database)
	.await?;

	let commands = get_guild_commands(context.data(), &guild).await?;
	register_in_guild(context, &commands, guild).await?;
	context.reply(format!("Enabled Module: {module:?}")).await?;

	Ok(())
}

#[command(owners_only, guild_only, slash_command)]
async fn list(context: Context<'_>) -> Result<()> {
	let id = Id::from(context.guild_id().expect("command is guild only"));
	let module_list = get_formatted_module_list(id, &context.data().database).await?;
	context.reply(format!("Enabled Modules: {module_list}.",)).await?;

	Ok(())
}

#[command(owners_only, guild_only, slash_command)]
async fn disable(context: Context<'_>, module: Module) -> Result<()> {
	let guild = context.guild_id().expect("command is guild only");
	let id = Id::from(guild);

	query!(
		"DELETE FROM modules WHERE guild = $1 AND module = $2",
		id as _,
		module as _
	)
	.execute(&context.data().database)
	.await?;

	let commands = get_guild_commands(context.data(), &guild).await?;
	register_in_guild(context, &commands, guild).await?;
	context.reply(format!("Disabled Module: {module:?}")).await?;

	Ok(())
}
