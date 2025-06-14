use alloy::primitives::{Address, U256, address, utils::parse_ether};
use color_eyre::eyre::Context as _;
use or_poisoned::OrPoisoned as _;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
	application::interaction::Interaction,
	http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;
use crate::prelude::*;

use crate::{
	chains::{BlockchainListing, SomniaShannon},
	commands::faucet::DiscordInfo,
	common::GlobalStateRef,
	ratelimits,
};

/// Faucet 0.05 PING on Somnia Shannon (an ERC20 token)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "somnia-shannon-ping")]
pub struct SomniaShannonPing {
	/// Your personal wallet address
	address: String,
}

impl BlockchainListing for SomniaShannonPing {
	fn chain_id(&self) -> u64 {
		self.plain().chain_id()
	}

	fn chain_name(&self) -> &'static str {
		self.plain().chain_name()
	}

	fn native_token_name(&self) -> &'static str {
		self.plain().native_token_name()
	}
}

impl SomniaShannonPing {
	fn plain(&self) -> SomniaShannon {
		SomniaShannon {
			address: self.address.clone(),
		}
	}

	fn amount() -> U256 {
		parse_ether("0.05").unwrap()
	}

	fn erc20_token_name() -> &'static str {
		"PING"
	}

	const SMART_CONTRACT_ADDR: Address = address!("0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493");
}

impl SomniaShannonPing {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		discord_info: DiscordInfo,
	) -> color_eyre::Result<()> {
		let address = &self.address;
		let chain_id = self.chain_id();
		let chain_name = self.chain_name();
		let DiscordInfo {
			discord_id,
			has_expanded_limits,
		} = discord_info;

		let respond = async |msg: &str| {
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
		};
		let follow_up = async |msg: &str| {
			state
				.client
				.interaction(interaction.application_id)
				.create_followup(&interaction.token)
				.content(msg.as_ref())
				.await
				.wrap_err("Couldn't followup a discord interaction")
		};

		let address = match address.parse() {
			Ok(address) => address,
			Err(err) => {
				respond(&format!(
					"Invalid Etherium wallet address {:?}: {}",
					self.address, err
				))
				.await?;
				return Ok(());
			}
		};

		// check ratelimiting if not expanded limits
		let ratelimit_key = ratelimits::Key {
			address,
			discord_id,
			chain_id,
			chain_name,
		};
		if !has_expanded_limits {
			let ratelimit = state.ratelimits.lock().or_poisoned().check(&ratelimit_key);
			if let Err(msg) = ratelimit {
				let msg = format!(
					"Couldn't faucet you any tokens because you are ratelimited!\n{}",
					msg
				);
				respond(&msg).await?;
				return Ok(());
			}
		} else {
			info!(%discord_id, "This person has expanded limits");
		}

		Ok(())
	}
}
