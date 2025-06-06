use std::sync::Mutex;

use crate::{chains, chains::SupportedChain, prelude::*, ratelimits::Key};
use chains::FaucetBlockchain as _;
use color_eyre::Section;
use salt_sdk::{Salt, SaltConfig, TransactionInfo};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::{Interaction, application_command::CommandData},
	http::interaction::{InteractionResponse, InteractionResponseType},
	id::Id,
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

mod check;

// /// Check all your ratelimits by chain
// #[derive(Debug, Clone, CommandModel, CreateCommand)]
// #[command(name = "ratelimits")]
// pub(super) struct CheckRatelimits {
// 	/// Your personal wallet address
// 	pub address: String,
// }

pub struct DiscordInfo {
	discord_id: u64,
	has_expanded_limits: bool,
}

async fn discord_info(
	state: GlobalStateRef<'_>,
	interaction: &Interaction,
) -> color_eyre::Result<DiscordInfo> {
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
	let expanded_limits_id = Id::new(1364832034677198949);
	let has_expanded_limits = member.roles.contains(&expanded_limits_id);
	let discord_id = user.id.get();
	Ok(DiscordInfo {
		discord_id,
		has_expanded_limits,
	})
}

impl FaucetCommand {
	pub async fn handle(
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		data: CommandData,
	) -> color_eyre::Result<()> {
		let command =
			FaucetCommand::from_interaction(data.into()).wrap_err("Couldn't parse command data")?;

		let discord_info = discord_info(state, &interaction).await?;
		let res = state.per_user_spam_filters.engage(discord_info.discord_id);
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
						.handle(state, interaction, discord_info)
						.await
				}
				FaucetCommand::SepoliaArbitrum(chain) => {
					SupportedChain::SepoliaArbitrum(chain)
						.handle(state, interaction, discord_info)
						.await
				}
				FaucetCommand::SepoliaEtherium(chain) => {
					SupportedChain::SepoliaEtherium(chain)
						.handle(state, interaction, discord_info)
						.await
				}
				FaucetCommand::SomniaShannon(chain) => {
					SupportedChain::SomniaShannon(chain)
						.handle(state, interaction, discord_info)
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
		discord_info: DiscordInfo,
	) -> color_eyre::Result<()> {
		let chains::BlockchainInfo {
			chain_id,
			rpc_url,
			token_name,
			chain_name,
		} = self.info(state.env);
		let amount = self.faucet_amount();
		let address = self.address();
		let DiscordInfo {
			discord_id,
			has_expanded_limits,
		} = discord_info;

		// check ratelimiting if not expanded limits
		let ratelimit_key = Key {
			address: address.clone().into_boxed_str(),
			discord_id: discord_id.to_string().into_boxed_str(),
			chain_id,
			chain_name: chain_name.to_owned(),
		};
		if !has_expanded_limits {
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
		} else {
			info!("This person has expanded limits");
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

		// initial response
		let response = InteractionResponse {
			kind: InteractionResponseType::ChannelMessageWithSource,
			data: Some(
				InteractionResponseDataBuilder::new()
					.content(&format!(
						"Starting faucet of {amount}{token_name} ({chain_name}) to {address} ..."
					))
					.build(),
			),
		};
		state
			.client
			.interaction(interaction.application_id)
			.create_response(interaction.id, &interaction.token, &response)
			.await
			.wrap_err("Unable to respond to the interaction initially")?;

		// do transaction
		let (send_logs, mut live_logs) = tokio::sync::mpsc::channel(10);
		let salt_config = SaltConfig {
			private_key: state.env.private_key.clone(),
			orchestration_network_rpc_node: state.env.orchestration_network_rpc_node_url.clone(),
			broadcasting_network_rpc_node: rpc_url,
			broadcasting_network_id: chain_id,
		};
		let salt = Salt::new(salt_config)?;
		let transaction = salt.transaction(TransactionInfo {
			amount: &amount,
			vault_address: &state.env.faucet_testnet_salt_account_address,
			recipient_address: &address,
			data: "",
			logging: send_logs,
		});
		let interaction2 = interaction.clone();
		let logging = async move {
			let interaction = interaction2;
			while let Some(log) = live_logs.recv().await {
				info!(%log, "Sending live log");
				state
					.client
					.interaction(interaction.application_id)
					.create_followup(&interaction.token)
					.content(&log)
					.await
					.wrap_err("Couldn't follow up with a live logging message")
					.note("Couldn't follow up discord interaction")?;
			}
			Result::<(), color_eyre::Report>::Ok(())
			// return salt_sdk::Error::LiveLogging(eyre!("Live logging disconnected"));
		};

		let (res, logging_err) = tokio::join!(transaction, logging);

		if let Err(err) = logging_err {
			error!("Failed to send live logs:\n{}", err);
		}

		if let Err(err) = res {
			error!("Failed to do salt transaction:\n{}", err);
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
			// still registers even if expanded limits
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
