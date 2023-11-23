use crate::{modules::Module, Context, Data, Error, Result};
use poise::{command, Command};

pub struct Test;

impl Test {
	#[command(slash_command)]
	pub async fn test(context: Context<'_>) -> Result<()> {
		context.reply("Test!").await?;
		Ok(())
	}
}

impl Module for Test {
	const NAME: &'static str = "test";

	fn append_commands(commands: &mut Vec<Command<Data, Error>>) {
		commands.push(Self::test());
	}
}
