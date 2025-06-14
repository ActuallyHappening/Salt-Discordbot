use alloy::primitives::{address, utils::parse_ether, Address, U256};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::Interaction;

use crate::{commands::faucet::DiscordInfo, common::GlobalStateRef, ratelimits};


/// Faucet 0.05 PING on Somnia Shannon (an ERC20 token)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "somnia-shannon-ping")]
pub struct SomniaShannonPing {
	address: String,
}

impl SomniaShannonPing {
	fn amount() -> U256 {
		parse_ether("0.05").unwrap()
	}

	const SMART_CONTRACT_ADDR: Address = address!("0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493");
}

impl SomniaShannonPing {
	pub async fn handle(&self, state: GlobalStateRef<'_>, interaction: Interaction, discord_info: DiscordInfo) -> color_eyre::Result<()> {
		let address = self.address;
		let DiscordInfo {
			discord_id,
			has_expanded_limits,
		} = discord_info;

		// check ratelimiting if not expanded limits
		let ratelimit_key = ratelimits::KeyBuilder {
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
			info!(discord_id, "This person has expanded limits");
		}
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
		Ok(())
	}
}
