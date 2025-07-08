use alloy::{
	primitives::utils::{ParseUnits, Unit},
	providers::{Provider as _, ProviderBuilder},
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::{Interaction, application_command::CommandData};

use crate::{
	commands::{defer, follow_up, respond},
	common::GlobalStateRef,
	prelude::*,
};

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
		defer(state, &interaction).await?;

		let provider = ProviderBuilder::new()
			.connect(state.env.somnia_shannon_rpc_endpoint.as_str())
			.await?;
		let account_addr = state.env.faucet_testnet_salt_account_address;

		let sst_balance =
			ParseUnits::from(provider.get_balance(account_addr).await?).format_units(Unit::ETHER);
		let erc20_addrs = [standard_sdk::USDC, standard_sdk::WSOL, standard_sdk::WBTC];
		let mut erc20_tokens = vec![];
		struct TokenInfo {
			name: String,
			symbol: String,
			#[allow(unused)]
			decimals: u8,
			balance: String,
		}
		for token in erc20_addrs {
			let token = ERC20::new(token, &provider);
			let balance = token.balanceOf(account_addr).call().await?;
			let decimals = token.decimals().call().await?;
			let balance = ParseUnits::from(balance).format_units(decimals.try_into()?);
			let name = token.name().call().await?;
			let symbol = token.symbol().call().await?;
			erc20_tokens.push(TokenInfo {
				name,
				symbol,
				decimals,
				balance,
			});
		}

		let mut msg = format!("The balance for the test Salt account ({account_addr}):\n");
		msg.push_str(&format!("Native SST: {sst_balance}\n"));
		for token in erc20_tokens {
			msg.push_str(&format!(
				"{} {}: {}\n",
				token.name, token.symbol, token.balance
			));
		}

		follow_up(state, &interaction, &msg).await?;

		Ok(())
	}
}

// Generate the contract bindings for the ERC20 interface.
alloy::sol! {
	// The `rpc` attribute enables contract interaction via the provider.
	#[sol(rpc, abi)]
	contract ERC20 {
		function name() public view returns (string);
		function symbol() public view returns (string);
		function decimals() public view returns (uint8);
		function totalSupply() public view returns (uint256);
		function balanceOf(address account) public view returns (uint256);
		function transfer(address recipient, uint256 amount) public returns (bool);
		function allowance(address owner, address spender) public view returns (uint256);
		function approve(address spender, uint256 amount) public returns (bool);
		function transferFrom(address sender, address recipient, uint256 amount) public returns (bool);

		event Transfer(address indexed from, address indexed to, uint256 value);
		event Approval(address indexed owner, address indexed spender, uint256 value);
	}
}
