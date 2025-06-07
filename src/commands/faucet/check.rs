use twilight_interactions::command::{CommandModel, CreateCommand};

use crate::common::GlobalStateRef;

/// Check which chains your wallet address is valid for,
/// including ratelimits
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "check")]
pub(super) struct Check {
	/// Your personal wallet address
	pub address: String,
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
