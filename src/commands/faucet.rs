use crate::{prelude::*, ratelimits::Key};
use salt_sdk::{Salt, SaltConfig};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::{Interaction, application_command::CommandData},
	http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::common::GlobalStateRef;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
	name = "salt-faucet",
	desc = "Faucet some crypto from a testing Salt account"
)]
pub(super) enum FaucetCommand {
	#[command(name = "somnia-shannon")]
	SomniaShannon(SomniaShannon),
	#[command(name = "sepolia-eth")]
	SepoliaEtherium(SepoliaEtherium),
	#[command(name = "sepolia-arb-eth")]
	SepoliaArbitrum(SepoliaArbitrum),
	#[command(name = "polygon-amoy")]
	PolygonAmoy(PolygonAmoy),
}

/// Faucet 0.01 on Somnia Shannon SST tokens
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "somnia-shannon")]
pub struct SomniaShannon {
	/// Your personal wallet address
	pub address: String,
}

/// Faucet 0.005ETH on Ethereum Sepolia
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-eth")]
pub struct SepoliaEtherium {
	/// Your personal wallet address
	pub address: String,
}

/// Faucet 0.005ETH on Arbitrum Sepolia (gas for salt orchestration)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-arb-eth")]
pub struct SepoliaArbitrum {
	/// Your personal wallet address
	pub address: String,
}

/// Faucet 0.005ETH on Polygon Amoy
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "polygon-amoy")]
pub struct PolygonAmoy {
	/// Your personal wallet address
	pub address: String,
}

impl FaucetCommand {
	pub async fn handle(
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		data: CommandData,
	) -> color_eyre::Result<()> {
		let command =
			FaucetCommand::from_interaction(data.into()).wrap_err("Couldn't parse command data")?;

		command.faucet(state, interaction).await
	}

	pub async fn faucet(
		self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		let member = match interaction.member {
			Some(user) => user,
			None => {
				let data = InteractionResponseDataBuilder::new()
					.content("Not called as a /slash command? `interaction.member` not received\nThis data is used to ratelimit discord users")
					.build();
				let response = InteractionResponse {
					kind: InteractionResponseType::ChannelMessageWithSource,
					data: Some(data),
				};
				state
					.client
					.interaction(interaction.application_id)
					.create_response(interaction.id, &interaction.token, &response)
					.await?;
				bail!("Must be provided a member");
			}
		};
		let user = match member.user {
			Some(user) => user,
			None => {
				let data = InteractionResponseDataBuilder::new()
					.content(
						"`interaction.member.user` was not received\nThis data is used to ratelimit discord users",
					)
					.build();
				let response = InteractionResponse {
					kind: InteractionResponseType::ChannelMessageWithSource,
					data: Some(data),
				};
				state
					.client
					.interaction(interaction.application_id)
					.create_response(interaction.id, &interaction.token, &response)
					.await?;
				bail!("Must be provided a user ID");
			}
		};
		let discord_id = user.id.to_string();

		let (chain_id, rpc_url, address, chain_name, token_name, amount) = match self {
			FaucetCommand::SepoliaArbitrum(data) => (
				421614,
				state.env.sepolia_arbitrum_rpc_endpoint.clone(),
				data.address,
				"ETH",
				"Sepolia Arbitrum",
				0.005,
			),
			FaucetCommand::SepoliaEtherium(data) => (
				11155111,
				state.env.sepolia_etherium_rpc_endpoint.clone(),
				data.address,
				"ETH",
				"Sepolia Ethereum",
				0.005,
			),
			FaucetCommand::SomniaShannon(data) => (
				50312,
				state.env.somnia_shannon_rpc_endpoint.clone(),
				data.address,
				"SST",
				"Somnia Shannon",
				0.01,
			),
			FaucetCommand::PolygonAmoy(data) => (
				80002,
				state.env.polygon_amoy_rpc_endpoint.clone(),
				data.address,
				"AMOY",
				"Polygon Amoy",
				0.005,
			),
		};

		// check ratelimiting
		let ratelimit_key = Key {
			address: address.clone().into_boxed_str(),
			discord_id: discord_id.into_boxed_str(),
			chain_id,
			chain_name: chain_name.to_owned(),
		};
		let ratelimit = state.ratelimits.lock().or_poisoned().check(&ratelimit_key);
		if let Err(msg) = ratelimit {
			let data = InteractionResponseDataBuilder::new()
				.content(format!(
					"Couldn't faucet you any tokens because you are ratelimited!\n{}",
					msg
				))
				.build();
			let response = InteractionResponse {
				kind: InteractionResponseType::ChannelMessageWithSource,
				data: Some(data),
			};
			state
				.client
				.interaction(interaction.application_id)
				.create_response(interaction.id, &interaction.token, &response)
				.await?;
			return Ok(());
		}

		// defer response
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

		let salt_config = SaltConfig {
			private_key: state.env.private_key.clone(),
			orchestration_network_rpc_node: state.env.orchestration_network_rpc_node_url.clone(),
			broadcasting_network_rpc_node: rpc_url,
			broadcasting_network_id: chain_id,
		};
		let salt = Salt::new(salt_config)?;
		let res = salt.transaction(
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
			err_string =
				format!("Error transacting {amount}{token_name} to {address}:\n{err_string}");
			state
				.client
				.interaction(interaction.application_id)
				.create_followup(&interaction.token)
				.content(&err_string)
				.await
				.wrap_err("Couldn't follow up on a failed transaction with an error message")?;
		} else {
			state
				.ratelimits
				.lock()
				.or_poisoned()
				.register(&ratelimit_key)
				.wrap_err("Couldn't register successful bot transaction")?;
			state
				.client
				.interaction(interaction.application_id)
				.create_followup(&interaction.token)
				.content(&format!(
					"Successful faucet of {amount}{token_name} ({chain_name}) to {address}"
				))
				.await
				.wrap_err("Couldn't follow up a successful transaction")?;
		}

		Ok(())
	}
}
