use alloy::providers::{Provider, ProviderBuilder};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

use crate::{common::GlobalStateRef, prelude::*};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
	name = "somnia-standard",
	desc = "Interact with the Somnia Standard on-chain trading platform"
)]
pub enum SomniaStandardCommand {
	#[command(name = "balance")]
	Balance(Balance),
}

/// Check the Standard balance of the Salt bot account
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "balance")]
pub struct Balance;

impl SomniaStandardCommand {
	pub async fn handle(
		state: GlobalStateRef<'_>,
		interaction: Interaction,
		data: CommandData,
	) -> color_eyre::Result<()> {
		let this = SomniaStandardCommand::from_interaction(data.into())
			.wrap_err("Couldn't parse command data")?;
		this._handle(state, interaction).await
	}

	async fn _handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		match self {
			SomniaStandardCommand::Balance(balance) => balance.handle(state, interaction).await,
		}
	}
}

impl Balance {
	pub async fn handle(
		&self,
		state: GlobalStateRef<'_>,
		interaction: Interaction,
	) -> color_eyre::Result<()> {
		let provider = ProviderBuilder::new()
			.connect(state.env.somnia_shannon_rpc_endpoint.as_str())
			.await?;
		let addr = state.env.faucet_testnet_salt_account_address;

		let sst_balance = provider.get_balance(addr).await?;

		Ok(())
	}
}
