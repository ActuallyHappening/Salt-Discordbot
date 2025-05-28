use crate::{prelude::*, ratelimits::Key};
use chains::FaucetBlockchain as _;
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

	impl FaucetBlockchain for FaucetCommand {
		fn info(&self, env: &Env) -> BlockchainInfo {
			match self {
				FaucetCommand::SomniaShannon(chain) => chain.info(env),
				FaucetCommand::PolygonAmoy(chain) => chain.info(env),
				FaucetCommand::SepoliaArbitrum(chain) => chain.info(env),
				FaucetCommand::SepoliaEtherium(chain) => chain.info(env),
			}
		}

		fn address(&self) -> String {
			match self {
				FaucetCommand::SomniaShannon(chain) => chain.address(),
				FaucetCommand::PolygonAmoy(chain) => chain.address(),
				FaucetCommand::SepoliaArbitrum(chain) => chain.address(),
				FaucetCommand::SepoliaEtherium(chain) => chain.address(),
			}
		}

		fn amount(&self) -> String {
			match self {
				FaucetCommand::SomniaShannon(chain) => chain.amount(),
				FaucetCommand::PolygonAmoy(chain) => chain.amount(),
				FaucetCommand::SepoliaArbitrum(chain) => chain.amount(),
				FaucetCommand::SepoliaEtherium(chain) => chain.amount(),
			}
		}
	}

	/// Faucet 0.01 on Somnia Shannon SST tokens
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
				token_name: "SST",
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
		let res = salt.transaction(
			&amount.to_string(),
			&state.env.salt_account_address,
			&address,
		);

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
