use crate::{common::GlobalState, prelude::*};
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

pub fn commands() -> Vec<twilight_model::application::command::Command> {
	[faucet::FaucetCommand::create_command().into()]
		.into_iter()
		.collect()
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
		name => bail!("unknown command: {}", name),
	}
}

mod faucet;

