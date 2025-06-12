use color_eyre::eyre::Context as _;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::{Interaction, application_command::CommandData},
	http::interaction::InteractionResponseType,
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::common::GlobalStateRef;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "admin", desc = "Admin commands for the Salt discord bot")]
pub(super) enum AdminCommand {
	#[command(name = "dump-user-dedupe")]
	DumpUserDedupe(DumpUserDedupe),

	#[command(name = "purge-user-dedupe")]
	PurgeUserDedupe(PurgeUserDedupe),
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
