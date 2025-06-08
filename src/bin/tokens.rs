#[allow(unused_imports)]
use ::tracing::{debug, error, info, trace, warn};
use alloy::{
	primitives::{
		address, utils::{parse_ether, ParseUnits, Unit}, Address, FixedBytes, Uint, U256
	},
	providers::ProviderBuilder,
	signers::{k256::sha2::digest::typenum::UInt, Signer},
	sol,
};
use color_eyre::eyre::Context as _;
use alloy::sol_types::SolCall;
use hex::prelude::*;

#[path = "../tracing.rs"]
mod app_tracing;

/// https://shannon-explorer.somnia.network/token/0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493
const PING: alloy::primitives::Address =
	alloy::primitives::address!("0x33E7fAB0a8a5da1A923180989bD617c9c2D1C493");
/// https://shannon-explorer.somnia.network/token/0x9beaA0016c22B646Ac311Ab171270B0ECf23098F
const PONG: alloy::primitives::Address =
	alloy::primitives::address!("0x9beaA0016c22B646Ac311Ab171270B0ECf23098F");

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,salt_sdk=trace,tokens=trace")?;
	trace!("Started tokens.rs");

	// Generate the contract bindings for the ERC20 interface.
	sol! {
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

	let to = address!("0xEA428233445A5Cf500B9d5c91BcA6E7B887f7D70");
	let amount = parse_ether("0.5")?;

	let env = salt_discordbot::env::Env::default()?;
	let provider = alloy::providers::ProviderBuilder::new().connect(env.somnia_shannon_rpc_endpoint.as_str()).await?;
	let call = ERC20::transferCall { amount, recipient: to };
	let data_str = call.abi_encode().to_lower_hex_string();

	// let somnia_rpc = env.somnia_shannon_rpc_endpoint;

	// // #[allow(unused)]
	// // let mut rl = rustyline::DefaultEditor::new()?;

	// {
	// 	let private_key = include_str!("private_key").trim();

	// 	let to = address!("0x85BCADfB48E95168b3C4aA3221ca2526CF96c99E");
	// 	let amount = ("0.5", Unit::ETHER);
	// 	let amount = alloy::primitives::utils::ParseUnits::parse_units(amount.0, amount.1)
	// 		.unwrap()
	// 		.get_absolute();

	// 	let mut signer: alloy::signers::local::PrivateKeySigner = private_key.parse()?;
	// 	signer.set_chain_id(Some(50312));
	// 	let me = signer.address();

	// 	let provider = alloy::providers::ProviderBuilder::new()
	// 		.wallet(signer)
	// 		.connect(somnia_rpc.as_str())
	// 		.await?;
	// 	let erc20_ping = ERC20::new(PING, provider.clone());

	// 	let balance = erc20_ping.balanceOf(me).call().await?;
	// 	let balance: ParseUnits = balance.into();
	// 	info!("My balance: {}", balance.format_units(Unit::ETHER));

	// 	let to_balance: ParseUnits = erc20_ping.balanceOf(to).call().await?.into();
	// 	info!("To balance: {}", to_balance.format_units(Unit::ETHER));

	// 	let total_balance: ParseUnits = erc20_ping.totalSupply().call().await?.into();
	// 	let symbol = erc20_ping.symbol().call().await?;
	// 	let name = erc20_ping.name().call().await?;
	// 	info!("total balance: {}, symbol: {}, name: {}", total_balance.format_units(Unit::ETHER), symbol, name);

	// 	let transfer_tx = erc20_ping.transfer(to, amount).send().await?;
	// 	let recipt = transfer_tx.get_receipt().await?;
	// 	info!("see on: https://shannon-explorer.somnia.network/tx/{}", recipt.transaction_hash);

	// 	let balance = erc20_ping.balanceOf(me).call().await?;
	// 	let balance: ParseUnits = balance.into();
	// 	info!("My balance: {}", balance.format_units(Unit::ETHER));

	// 	let to_balance: ParseUnits = erc20_ping.balanceOf(to).call().await?.into();
	// 	info!("To balance: {}", to_balance.format_units(Unit::ETHER));
	// }

	Ok(())
}
