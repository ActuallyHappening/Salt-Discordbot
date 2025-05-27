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

mod faucet {
	use crate::prelude::*;
	use salt_sdk::{Salt, SaltConfig};
	use twilight_interactions::command::{CommandModel, CreateCommand};
	use twilight_model::{
		application::interaction::{Interaction, application_command::CommandData},
		http::interaction::{
			InteractionResponse, InteractionResponseType,
		},
	};

	use crate::common::GlobalStateRef;

	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(
		name = "salt-faucet",
		desc = "Faucet some crypto from a testing Salt account"
	)]
	pub(super) enum FaucetCommand {
		#[command(name = "somnia-shannon")]
		SomniaShannon(SomniaShannon),
		#[command(name = "sepolia-etherium")]
		SepoliaEtherium(SepoliaEtherium),
		#[command(name = "sepolia-arbitrum")]
		SepoliaArbitrum(SepoliaArbitrum),
	}

	/// Faucet 0.01 Somnia Shannon SST tokens
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "somnia-shannon")]
	pub struct SomniaShannon {
		/// Your personal wallet address
		pub address: String,
	}

	/// Faucet 0.01ETH Sepolia
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "sepolia-etherium")]
	pub struct SepoliaEtherium {
		/// Your personal wallet address
		pub address: String,
	}

	/// Faucet 0.01ETH Arbitrum Sepolia (gas for salt orchestration)
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "sepolia-arbitrum")]
	pub struct SepoliaArbitrum {
		/// Your personal wallet address
		pub address: String,
	}

	impl FaucetCommand {
		pub async fn handle(
			state: GlobalStateRef<'_>,
			interaction: Interaction,
			data: CommandData,
		) -> color_eyre::Result<()> {
			let command = FaucetCommand::from_interaction(data.into())
				.wrap_err("Couldn't parse command data")?;

			command.faucet(state, interaction).await
		}

		pub async fn faucet(
			self,
			state: GlobalStateRef<'_>,
			interaction: Interaction,
		) -> color_eyre::Result<()> {
			let (chain_id, rpc_url, address, token_name) = match self {
				FaucetCommand::SepoliaArbitrum(data) => (
					421614,
					state.env.sepolia_arbitrum_rpc_endpoint.clone(),
					data.address,
					"ETH (sepolia arbiturm)",
				),
				FaucetCommand::SepoliaEtherium(data) => (
					11155111,
					state.env.sepolia_etherium_rpc_endpoint.clone(),
					data.address,
					"ETH (sepolia etherium)",
				),
				FaucetCommand::SomniaShannon(data) => (
					50312,
					state.env.somnia_shannon_rpc_endpoint.clone(),
					data.address,
					"SST (somnia shannon)",
				),
			};
			let salt_config = SaltConfig {
				private_key: state.env.private_key.clone(),
				orchestration_network_rpc_node: state
					.env
					.orchestration_network_rpc_node_url
					.clone(),
				broadcasting_network_rpc_node: rpc_url,
				broadcasting_network_id: chain_id,
			};
			let salt = Salt::new(salt_config)?;

			let response = InteractionResponse {
				kind: InteractionResponseType::DeferredChannelMessageWithSource,
				data: None,
			};
			state
				.client
				.interaction(interaction.application_id)
				.create_response(interaction.id, &interaction.token, &response)
				.await
				.wrap_err("Unable to mark interaction as deferred")?;

			let amount = 0.01;
			let res = salt
				.transaction(
					&amount.to_string(),
					&state.env.salt_account_address,
					&address,
				);

			if let Err(err) = res {
				let mut err_string = err.to_string();
				if err_string.len() > 1900 {
					// only keeps first 1900 bytes, avoiding a panic if using String.split_off
					// https://doc.rust-lang.org/stable/std/string/struct.String.html#method.split_off
					err_string = String::from_utf8_lossy(
						&err_string
							.into_bytes()
							.into_iter()
							.take(1900)
							.collect::<Vec<u8>>(),
					)
					.into();
				}
				err_string = format!(
					"Error transacting {amount}{token_name} to {address}:\n{err_string}"
				);
				state
					.client
					.interaction(interaction.application_id)
					.create_followup(&interaction.token)
					.content(&err_string)
					.await
					.wrap_err("Couldn't follow up on a failed transaction with an error message")?;
			} else {
				state
					.client
					.interaction(interaction.application_id)
					.create_followup(&interaction.token)
					.content(&format!(
						"Successfully faucetted {amount}{token_name} to {address}"
					))
					.await
					.wrap_err("Couldn't follow up a successful transaction")?;
			}

			Ok(())
		}
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
