#[allow(unused_imports)]
use ::tracing::{debug, error, info, trace, warn};
use alloy::{
	primitives::{
		Address, U256,
		utils::{ParseUnits, Unit, parse_ether},
	},
	signers::local::PrivateKeySigner,
	sol_types::SolCall as _,
};
use clap::Parser;
use color_eyre::eyre::Context as _;
use hex::DisplayHex as _;

#[path = "../tracing.rs"]
mod app_tracing;

/// https://shannon-explorer.somnia.network/token/0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493
const PING: alloy::primitives::Address =
	alloy::primitives::address!("0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493");

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

#[derive(clap::Parser)]
pub enum Cli {
	Read,
	Do,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,salt_sdk=debug,tokens=trace")?;
	trace!("Started tokens.rs");

	let cli = Cli::parse();

	let env = salt_discordbot::env::Env::default()?;
	let signer: PrivateKeySigner = env.private_key.parse()?;
	let me = signer.address();
	let provider = alloy::providers::ProviderBuilder::new()
		.wallet(signer)
		.connect(env.somnia_shannon_rpc_endpoint.as_str())
		.await?;

	let sanity_check = async || {
		// check balances for sanity
		let salt_wallet: Address = env.faucet_testnet_salt_account_address;
		let provider = alloy::providers::ProviderBuilder::new()
			.connect(env.somnia_shannon_rpc_endpoint.as_str())
			.await?;
		let personal_balance: ParseUnits = ERC20::new(PING, provider.clone())
			.balanceOf(me)
			.call()
			.await?
			.into();
		info!(
			"Personal balance of PING: {}",
			personal_balance.format_units(Unit::ETHER)
		);
		let salt_balance: ParseUnits = ERC20::new(PING, provider)
			.balanceOf(salt_wallet)
			.call()
			.await?
			.into();
		info!(
			"Salt balance of PING: {}",
			salt_balance.format_units(Unit::ETHER)
		);
		color_eyre::Result::<(), color_eyre::Report>::Ok(())
	};

	sanity_check().await?;

	if matches!(cli, Cli::Do) {
		// A salt token transfer
		let amount = parse_ether("0.5")?;
		let call = ERC20::transferCall {
			amount,
			recipient: me,
		};
		let calldata = call.abi_encode();

		let salt = salt_sdk::Salt::new(salt_sdk::SaltConfig {
			private_key: env.private_key,
			orchestration_network_rpc_node: env.sepolia_arbitrum_rpc_endpoint,
			broadcasting_network_rpc_node: env.somnia_shannon_rpc_endpoint.clone(),
			broadcasting_network_id: 50312,
		})?;
		let output = salt
			.transaction(salt_sdk::TransactionInfo {
				amount: U256::from(0),
				vault_address: env.faucet_testnet_salt_account_address,
				recipient_address: PING,
				data: calldata,
				logging: salt_sdk::LiveLogging::from_cb(|msg| info!(%msg, "Transaction live logs")),
				gas: salt_sdk::GasEstimator::Mul(100.0),
				confirm_publish: true,
			})
			.await
			.wrap_err("Unable to send transaction")?;
		info!(%output, "Done salt token transaction!");
	}

	loop {
		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
		sanity_check().await?;
	}

	Ok(())
}
