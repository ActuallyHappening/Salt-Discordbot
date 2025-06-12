use camino::Utf8PathBuf;
use color_eyre::eyre::Context as _;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::{Interaction, application_command::CommandData},
	http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;
use crate::prelude::*;

use crate::common::GlobalStateRef;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "admin", desc = "Admin commands for the Salt discord bot")]
pub(super) enum AdminCommand {
	#[command(name = "dump-user-dedupe")]
	DumpUserDedupe(DumpUserDedupe),

	#[command(name = "purge-user-dedupe")]
	PurgeUserDedupe(PurgeUserDedupe),

	#[command(name = "dump-logs")]
	DumpLogs(DumpLogs),
}

impl AdminCommand {
	pub async fn handle(
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		data: CommandData,
	) -> color_eyre::Result<()> {
		let command =
			AdminCommand::from_interaction(data.into()).wrap_err("Couldn't parse command data")?;

		match command {
			AdminCommand::DumpUserDedupe(_) => {
				let dump = state.per_user_spam_filters.dump();
				state
					.client
					.interaction(interaction.application_id)
					.create_response(
						interaction.id,
						&interaction.token,
						&twilight_model::http::interaction::InteractionResponse {
							data: Some(InteractionResponseDataBuilder::new().content(dump).build()),
							kind: InteractionResponseType::ChannelMessageWithSource,
						},
					)
					.await?;
				Ok(())
			}
			AdminCommand::PurgeUserDedupe(_) => {
				let dump = state.per_user_spam_filters.dump();
				state.per_user_spam_filters.purge();
				let msg = format!(
					"Purged the user dedupe data, this was what was purged:\n{}",
					dump
				);
				state
					.client
					.interaction(interaction.application_id)
					.create_response(
						interaction.id,
						&interaction.token,
						&twilight_model::http::interaction::InteractionResponse {
							kind: InteractionResponseType::ChannelMessageWithSource,
							data: Some(InteractionResponseDataBuilder::new().content(msg).build()),
						},
					)
					.await?;
				Ok(())
			}
			AdminCommand::DumpLogs(cmd) => {
				cmd.handle(state, interaction).await;
				Ok(())
			}
		}
	}
}

/// Dump the live user dedupe data into chat
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "dump-user-dedupe")]
pub(super) struct DumpUserDedupe;

/// Purge all user dedupe data, which may allow somebody to have two parallel requests
/// execute at the same time
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "purge-user-dedupe")]
pub(super) struct PurgeUserDedupe;

/// Dumps today's logs as a file
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "dump-logs")]
pub(super) struct DumpLogs;

impl DumpLogs {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		state
			.client
			.interaction(interaction.application_id)
			.create_response(
				interaction.id,
				&interaction.token,
				&InteractionResponse {
					kind: InteractionResponseType::DeferredChannelMessageWithSource,
					data: None,
				},
			)
			.await?;

		match self.get_file().await {
			Ok(file) => {
				todo!("Send file");
				Ok(())
			}
			Err(err) => {
				error!(%err, "Internal error while dumping logs");
				state
					.client
					.interaction(interaction.application_id)
					.create_followup(&interaction.token)
					.content(&format!(
						"An internal error occurred while dumping the logs: {}",
						err
					))
					.await?;
				Ok(())
			}
		}
	}

	async fn get_file(
		&self,
	) -> color_eyre::Result<Utf8PathBuf> {
		let now = time::OffsetDateTime::now_utc();
		let format = time::macros::format_description!("rust-discordbot.json.[year]-[month]-[day]");
		let file_name = now.format(&format);
		todo!()
	}
}
