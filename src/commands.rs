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
	info!(?data, ?interaction);
	match &*data.name {
		// "orders-list" => orders::OrdersListCommand::handle(state.clone(), interaction, data).await,
		name => bail!("unknown command: {}", name),
	}
}

mod faucet {
	use twilight_interactions::command::{CommandModel, CreateCommand};

	use crate::common::GlobalStateRef;

	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "salt-faucet", desc = "Faucet some crypto bro FIXME")]
	pub(super) struct FaucetCommand {
		#[command(desc = "Recipient wallet address")]
		recipient_address: String,
	}

	impl FaucetCommand {
		pub async fn faucet(state: GlobalStateRef<'_>) {}
	}
}

// mod orders {
// 	use crate::{
// 		common::{GlobalState, GlobalStateRef, to_string_pretty},
// 		prelude::*,
// 	};

// 	use twilight_interactions::command::{CommandModel, CreateCommand};
// 	use twilight_model::{
// 		application::interaction::{Interaction, application_command::CommandData},
// 		http::interaction::{
// 			InteractionResponse, InteractionResponseData, InteractionResponseType,
// 		},
// 	};

// 	#[derive(Debug, Clone, CommandModel, CreateCommand)]
// 	#[command(name = "orders-list", desc = "List all the current orders")]
// 	pub(super) struct OrdersListCommand;

// 	pub async fn list_orders(state: GlobalStateRef<'_>) -> String {
// 		state
// 			.db
// 			.orders()
// 			.select_fetch_user()
// 			.initial()
// 			.await
// 			.wrap_err("Couldn't fetch orders + users")
// 			.map(|orders| to_string_pretty(&orders))
// 			.unwrap_or_else(|err| format!("{:?}", err))
// 	}

// 	/// https://github.com/baptiste0928/twilight-interactions/blob/main/examples/xkcd-bot/interactions/command.rs
// 	impl OrdersListCommand {
// 		pub async fn handle(
// 			state: GlobalState,
// 			interaction: Interaction,
// 			data: CommandData,
// 		) -> Result<()> {
// 			// Parse the command data into a structure using twilight-interactions.
// 			let command =
// 				Self::from_interaction(data.into()).wrap_err("failed to parse command data")?;

// 			command.run(state, interaction).await?;

// 			Ok(())
// 		}

// 		pub async fn run(&self, state: GlobalState, interaction: Interaction) -> Result<()> {
// 			info!(?self, "Handling orders-list command");

// 			let content = list_orders(state.get()).await;
// 			debug!(?content);
// 			let data = InteractionResponseData {
// 				content: Some(content),
// 				..Default::default()
// 			};

// 			let client = state.get().client.interaction(interaction.application_id);
// 			let response = InteractionResponse {
// 				kind: InteractionResponseType::ChannelMessageWithSource,
// 				data: Some(data),
// 			};

// 			client
// 				.create_response(interaction.id, &interaction.token, &response)
// 				.await?;

// 			Ok(())
// 		}
// 	}
// }
// 
