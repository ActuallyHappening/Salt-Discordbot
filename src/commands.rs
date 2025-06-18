use crate::{
	common::{GlobalState, GlobalStateRef},
	prelude::*,
};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
	application::interaction::{Interaction, application_command::CommandData},
	http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub fn public_commands() -> Vec<twilight_model::application::command::Command> {
	[faucet::FaucetCommand::create_command().into()]
		.into_iter()
		.collect()
}

/// Includes all of [public_commands] as well
pub fn admin_commands() -> Vec<twilight_model::application::command::Command> {
	// let mut public = public_commands();
	let mut public = vec![];
	public.extend([
		admin::AdminCommand::create_command().into(),
		standard::SomniaStandardCommand::create_command().into(),
	]);
	public
}

/// Handle a command interaction.
pub async fn handle_command(
	state: GlobalState,
	interaction: Interaction,
	data: CommandData,
) -> Result<()> {
	trace!("Handling command interaction: {:#?}", interaction);
	match &*data.name {
		// "orders-list" => orders::OrdersListCommand::handle(state.clone(), interaction, data).await,
		"salt-faucet" => faucet::FaucetCommand::handle(state.get(), interaction, data).await,
		"admin" => admin::AdminCommand::handle(state.get(), interaction, data).await,
		"somnia-standard" => {
			standard::SomniaStandardCommand::handle(state.get(), interaction, data).await
		}
		name => bail!("unknown command: {}", name),
	}
}

mod admin;
mod faucet;
mod standard;

async fn defer(state: GlobalStateRef<'_>, interaction: &Interaction) -> color_eyre::Result<()> {
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
		.await
		.wrap_err("Couldn't initially respond to a discord interaction")
		.map(|_| ())
}

async fn respond(
	state: GlobalStateRef<'_>,
	interaction: &Interaction,
	msg: &str,
) -> color_eyre::Result<()> {
	state
		.client
		.interaction(interaction.application_id)
		.create_response(
			interaction.id,
			&interaction.token,
			&InteractionResponse {
				kind: InteractionResponseType::ChannelMessageWithSource,
				data: Some(InteractionResponseDataBuilder::new().content(msg).build()),
			},
		)
		.await
		.wrap_err("Couldn't initially respond to a discord interaction")
		.map(|_| ())
}

async fn follow_up(
	state: GlobalStateRef<'_>,
	interaction: &Interaction,
	msg: &str,
) -> color_eyre::Result<()> {
	state
		.client
		.interaction(interaction.application_id)
		.create_followup(&interaction.token)
		.content(msg.as_ref())
		.await
		.wrap_err("Couldn't followup a discord interaction").map(|_| ())
}
