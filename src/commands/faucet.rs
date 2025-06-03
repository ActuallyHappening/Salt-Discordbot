use crate::{commands::faucet::chains::SupportedChain, prelude::*, ratelimits::Key};
use chains::FaucetBlockchain as _;
use color_eyre::Section;
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
	SomniaShannon(chains::SomniaShannon),

	#[command(name = "sepolia-eth")]
	SepoliaEtherium(chains::SepoliaEtherium),

	#[command(name = "sepolia-arb-eth")]
	SepoliaArbitrum(chains::SepoliaArbitrum),

	#[command(name = "polygon-amoy")]
	PolygonAmoy(chains::PolygonAmoy),

	#[command(name = "check")]
	Check(Check),

	#[command(name = "ratelimits")]
	Ratelimits(CheckRatelimits),
}

/// Check which chains your wallet address is valid for,
/// including ratelimits
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "check")]
pub(super) struct Check {
	/// Your personal wallet address
	pub address: String,
}

/// Check all your ratelimits by chain
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "ratelimits")]
pub(super) struct CheckRatelimits {
	/// Your personal wallet address
	pub address: String,
}

mod chains {
	use twilight_interactions::command::{CommandModel, CreateCommand};
	use url::Url;

	use crate::{env::Env, prelude::*};

	use super::FaucetCommand;

	pub(super) trait FaucetBlockchain {
		fn info(&self, env: &Env) -> BlockchainInfo;

		fn amount(&self) -> String {
			String::from("0.005")
		}

		fn address(&self) -> String;
	}

	pub(super) struct BlockchainInfo {
		pub chain_id: u64,
		pub rpc_url: Url,
		pub token_name: &'static str,
		pub chain_name: &'static str,
	}

	#[derive(Debug, Clone)]
	pub(super) enum SupportedChain {
		SomniaShannon(SomniaShannon),
		SepoliaEtherium(SepoliaEtherium),
		SepoliaArbitrum(SepoliaArbitrum),
		PolygonAmoy(PolygonAmoy),
	}

	impl FaucetBlockchain for SupportedChain {
		fn info(&self, env: &Env) -> BlockchainInfo {
			match self {
				SupportedChain::SomniaShannon(chain) => chain.info(env),
				SupportedChain::PolygonAmoy(chain) => chain.info(env),
				SupportedChain::SepoliaArbitrum(chain) => chain.info(env),
				SupportedChain::SepoliaEtherium(chain) => chain.info(env),
			}
		}

		fn address(&self) -> String {
			match self {
				SupportedChain::SomniaShannon(chain) => chain.address(),
				SupportedChain::PolygonAmoy(chain) => chain.address(),
				SupportedChain::SepoliaArbitrum(chain) => chain.address(),
				SupportedChain::SepoliaEtherium(chain) => chain.address(),
			}
		}

		fn amount(&self) -> String {
			match self {
				SupportedChain::SomniaShannon(chain) => chain.amount(),
				SupportedChain::PolygonAmoy(chain) => chain.amount(),
				SupportedChain::SepoliaArbitrum(chain) => chain.amount(),
				SupportedChain::SepoliaEtherium(chain) => chain.amount(),
			}
		}
	}

	/// Faucet 0.01 on Somnia Shannon STT tokens
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "somnia-shannon")]
	pub struct SomniaShannon {
		/// Your personal wallet address
		pub address: String,
	}

	impl FaucetBlockchain for SomniaShannon {
		fn info(&self, env: &Env) -> BlockchainInfo {
			BlockchainInfo {
				chain_id: 50312,
				rpc_url: env.somnia_shannon_rpc_endpoint.clone(),
				token_name: "STT",
				chain_name: "Somnia Shannon",
			}
		}

		fn amount(&self) -> String {
			String::from("0.01")
		}

		fn address(&self) -> String {
			self.address.clone()
		}
	}

	/// Faucet 0.005ETH on Ethereum Sepolia
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "sepolia-eth")]
	pub struct SepoliaEtherium {
		/// Your personal wallet address
		pub address: String,
	}

	impl FaucetBlockchain for SepoliaEtherium {
		fn info(&self, env: &Env) -> BlockchainInfo {
			BlockchainInfo {
				chain_id: 11155111,
				rpc_url: env.sepolia_etherium_rpc_endpoint.clone(),
				token_name: "ETH",
				chain_name: "Sepolia Ethereum",
			}
		}

		fn address(&self) -> String {
			self.address.clone()
		}
	}

	/// Faucet 0.005ETH on Arbitrum Sepolia (gas for salt orchestration)
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "sepolia-arb-eth")]
	pub struct SepoliaArbitrum {
		/// Your personal wallet address
		pub address: String,
	}

	impl FaucetBlockchain for SepoliaArbitrum {
		fn info(&self, env: &Env) -> BlockchainInfo {
			BlockchainInfo {
				chain_id: 421614,
				rpc_url: env.sepolia_arbitrum_rpc_endpoint.clone(),
				token_name: "ETH",
				chain_name: "Sepolia Arbitrum",
			}
		}

		fn address(&self) -> String {
			self.address.clone()
		}
	}

	/// Faucet 0.005ETH on Polygon Amoy
	#[derive(Debug, Clone, CommandModel, CreateCommand)]
	#[command(name = "polygon-amoy")]
	pub struct PolygonAmoy {
		/// Your personal wallet address
		pub address: String,
	}

	impl FaucetBlockchain for PolygonAmoy {
		fn info(&self, env: &Env) -> BlockchainInfo {
			BlockchainInfo {
				chain_id: 80002,
				rpc_url: env.polygon_amoy_rpc_endpoint.clone(),
				token_name: "AMOY",
				chain_name: "Polygon Amoy",
			}
		}

		fn address(&self) -> String {
			self.address.clone()
		}
	}
}

async fn discord_user_id(
	state: GlobalStateRef<'_>,
	interaction: Interaction,
) -> color_eyre::Result<String> {
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
	Ok(discord_id)
}

impl FaucetCommand {
	pub async fn handle(
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		data: CommandData,
	) -> color_eyre::Result<()> {
		let command =
			FaucetCommand::from_interaction(data.into()).wrap_err("Couldn't parse command data")?;

		let res = {
			let state = state.reborrow();
			let interaction = interaction.clone();
			match command {
				FaucetCommand::Check(check) => check.handle(state, interaction).await,
				FaucetCommand::Ratelimits(ratelimits) => {
					ratelimits.handle(state, interaction).await
				}
				FaucetCommand::PolygonAmoy(chain) => {
					SupportedChain::PolygonAmoy(chain)
						.handle(state, interaction)
						.await
				}
				FaucetCommand::SepoliaArbitrum(chain) => {
					SupportedChain::SepoliaArbitrum(chain)
						.handle(state, interaction)
						.await
				}
				FaucetCommand::SepoliaEtherium(chain) => {
					SupportedChain::SepoliaEtherium(chain)
						.handle(state, interaction)
						.await
				}
				FaucetCommand::SomniaShannon(chain) => {
					SupportedChain::SomniaShannon(chain)
						.handle(state, interaction)
						.await
				}
			}
		};
		// global internal error handler
		// this is a best effort attempt, you must manually handle all user facing errors in .handle
		if let Err(err) = res {
			state
				.client
				.interaction(interaction.application_id)
				.create_followup(&interaction.token)
				.content(&format!("An internal error occurred: {}", err))
				.await
				.wrap_err("Couldn't send internal error message")
				.note(format!("Original internal error: {}", err))?;
		}
		Ok(())
	}
}

impl SupportedChain {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		let discord_id = discord_user_id(state, interaction).await?;
		let chains::BlockchainInfo {
			chain_id,
			rpc_url,
			token_name,
			chain_name,
		} = self.info(state.env);
		let amount = self.amount();
		let address = self.address();

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

		// do business logic checks
		match self {
			SupportedChain::SepoliaArbitrum(chain) => {
				if (!state.private_apis.test_1(&chain.address).await?) {

				}
			}
			_ => todo!(),
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

		// do transaction
		let salt_config = SaltConfig {
			private_key: state.env.private_key.clone(),
			orchestration_network_rpc_node: state.env.orchestration_network_rpc_node_url.clone(),
			broadcasting_network_rpc_node: rpc_url,
			broadcasting_network_id: chain_id,
		};
		let salt = Salt::new(salt_config)?;
		let res = salt
			.transaction(
				&amount.to_string(),
				&state.env.salt_account_address,
				&address,
			)
			.await;

		if let Err(err) = res {
			error!(?err, "Failed to do salt transaction");
			let mut err_string = err.to_string();

			if let salt_sdk::Error::SubprocessExitedBadlyWithOutput(output) = err {
				err_string = output.stderr;
			}

			if err_string.len() > 1900 {
				// only keeps first 1900 bytes, avoiding a panic if using String.split_off
				// https://doc.rust-lang.org/stable/std/string/struct.String.html#method.split_off
				let truncated = err_string
					.into_bytes()
					.into_iter()
					.take(1900)
					.collect::<Vec<u8>>();
				let truncated = String::from_utf8_lossy(&truncated);
				err_string = format!("{}...<truncated>", truncated);
			}
			err_string = format!(
				"Error transacting {amount}{token_name} ({chain_name}) to {address}:\n{err_string}"
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

impl Check {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		todo!()
	}

	/// Used on arb eth
	pub async fn test_1(address: String, state: GlobalStateRef<'_>) -> Result<(), CheckError> {
		todo!()
	}
}

#[derive(thiserror::Error, Debug)]
pub struct CheckError {
	address: String
}

impl CheckRatelimits {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		todo!()
	}
}
