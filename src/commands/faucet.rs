use crate::{chains, chains::SupportedChain, prelude::*, ratelimits::Key};
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
	// #[command(name = "check")]
	// Check(Check),

	// #[command(name = "ratelimits")]
	// Ratelimits(CheckRatelimits),
}

/// Check which chains your wallet address is valid for,
/// including ratelimits
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "check")]
pub(super) struct Check {
	/// Your personal wallet address
	pub address: String,
}

// /// Check all your ratelimits by chain
// #[derive(Debug, Clone, CommandModel, CreateCommand)]
// #[command(name = "ratelimits")]
// pub(super) struct CheckRatelimits {
// 	/// Your personal wallet address
// 	pub address: String,
// }

async fn discord_user_id(
	state: GlobalStateRef<'_>,
	interaction: &Interaction,
) -> color_eyre::Result<u64> {
	let member = match &interaction.member {
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
	let user = match &member.user {
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
	let discord_id = user.id.get();
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

		let discord_id = discord_user_id(state, &interaction).await?;
		let res = state.per_user_spam_filters.engage(discord_id);
		let _guard;
		match res {
			Err(err) => {
				let data = InteractionResponseDataBuilder::new()
					.content(err.to_string())
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
			Ok(guard) => {
				_guard = guard;
			}
		};

		let res = {
			let state = state.reborrow();
			let interaction = interaction.clone();
			match command {
				// FaucetCommand::Check(check) => check.handle(state, interaction).await,
				// FaucetCommand::Ratelimits(ratelimits) => {
				// 	ratelimits.handle(state, interaction).await
				// }
				FaucetCommand::PolygonAmoy(chain) => {
					SupportedChain::PolygonAmoy(chain)
						.handle(state, interaction, discord_id)
						.await
				}
				FaucetCommand::SepoliaArbitrum(chain) => {
					SupportedChain::SepoliaArbitrum(chain)
						.handle(state, interaction, discord_id)
						.await
				}
				FaucetCommand::SepoliaEtherium(chain) => {
					SupportedChain::SepoliaEtherium(chain)
						.handle(state, interaction, discord_id)
						.await
				}
				FaucetCommand::SomniaShannon(chain) => {
					SupportedChain::SomniaShannon(chain)
						.handle(state, interaction, discord_id)
						.await
				}
			}
		};
		// global internal error handler
		// this is a best effort attempt, you must manually handle all user facing errors in .handle
		if let Err(err) = res {
			error!(%err, ?err, "An internal error occurred");
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
		discord_id: u64,
	) -> color_eyre::Result<()> {
		let chains::BlockchainInfo {
			chain_id,
			rpc_url,
			token_name,
			chain_name,
		} = self.info(state.env);
		let amount = self.faucet_amount();
		let address = self.address();

		// check ratelimiting
		let ratelimit_key = Key {
			address: address.clone().into_boxed_str(),
			discord_id: discord_id.to_string().into_boxed_str(),
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
		// let check = Check { address: self.address() };
		// match self {
		// 	SupportedChain::SepoliaArbitrum(_) => {
		// 		if let Err(err) = check.test_1(state).await {
		// 			// handle error
		// 			let response = InteractionResponse {
		// 				kind: InteractionResponseType::ChannelMessageWithSource,
		// 				data: Some(
		// 					InteractionResponseDataBuilder::new()
		// 						.content(err.to_string())
		// 						.build(),
		// 				),
		// 			};
		// 			state
		// 				.client
		// 				.interaction(interaction.application_id)
		// 				.create_response(interaction.id, &interaction.token, &response)
		// 				.await?;
		// 			return Ok(())
		// 		}
		// 	}
		// 	_ => {
		// 		if let Err(err) = check.test_2(state).await {
		// 			// handle error
		// 			let response = InteractionResponse {
		// 				kind: InteractionResponseType::ChannelMessageWithSource,
		// 				data: Some(
		// 					InteractionResponseDataBuilder::new()
		// 						.content(err.to_string())
		// 						.build(),
		// 				),
		// 			};
		// 			state
		// 				.client
		// 				.interaction(interaction.application_id)
		// 				.create_response(interaction.id, &interaction.token, &response)
		// 				.await?;
		// 			return Ok(())
		// 		}
		// 	}
		// }

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
	// pub async fn handle(
	// 	&self,
	// 	state: GlobalStateRef<'_>,
	// 	interaction: Interaction,
	// ) -> color_eyre::Result<()> {
	// 	todo!()
	// }

	/// Used on arb eth only
	pub async fn test_1(&self, state: GlobalStateRef<'_>) -> Result<(), CheckError> {
		if !state
			.private_apis
			.test_1(&self.address)
			.await
			.map_err(CheckError::Inner)?
		{
			Err(CheckError::Test1 {
				address: self.address.clone(),
			})
		} else {
			Ok(())
		}
	}

	/// Used on all other chains than arb eth
	pub async fn test_2(&self, state: GlobalStateRef<'_>) -> Result<(), CheckError> {
		if !state
			.private_apis
			.test_2(&self.address)
			.await
			.map_err(CheckError::Inner)?
		{
			Err(CheckError::Test2 {
				address: self.address.clone(),
			})
		} else {
			Ok(())
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum CheckError {
	#[error(
		"You must belong to a Salt organisation to use this faucet! It's very easy to set up at https://testnet.salt.space - then return here to faucet Arbitrum ETH to use as gas to create your dMPC accounts"
	)]
	Test1 { address: String },

	#[error(
		"You must be a co-signer on an account on Salt to use this faucet! Invite someone to huddle with you and create your free dMPC accounts at https://testnet.salt.space - then return here to faucet some funds into it"
	)]
	Test2 { address: String },

	#[error("An internal error occurred looking up the Salt status of your wallet address: {0}")]
	Inner(#[source] color_eyre::Report),
}
