use alloy::{
	primitives::utils::{ParseUnits, Unit, parse_ether},
	providers::{Provider, ProviderBuilder},
	signers::local::PrivateKeySigner,
};
use standard_sdk::{
	USDC,
	abis::matching_engine::MatchingEngine::{cancelOrdersCall, marketSellETHCall},
};
use tracing::{debug, error, info, trace, warn};

#[path = "tracing.rs"]
mod app_tracing;

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

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,standard=trace")?;
	trace!("Started tracing");

	let private_key = include_str!("private-key");
	let signer: PrivateKeySigner = private_key.parse()?;
	let me = signer.address();
	let provider = ProviderBuilder::new()
		.wallet(signer.clone())
		.connect("https://dream-rpc.somnia.network/")
		.await?;

	let sst = ParseUnits::from(provider.get_balance(me).await?).format_units(Unit::ETHER);
	info!(sst, "My native SST balance pre");

	let usdc = ERC20::new(standard_sdk::USDC, &provider);
	let name = usdc.name().call().await?;
	let my_balance = usdc.balanceOf(me).call().await?;
	info!(?name, ?my_balance, "My USDC balance pre");

	let matching_engine = standard_sdk::abis::matching_engine::MatchingEngine::new(
		standard_sdk::abis::matching_engine::CONTRACT_ADDRESS,
		&provider,
	);
	{
		let tx = marketSellETHCall {
			quote: USDC,
			isMaker: true,
			n: 20,
			recipient: me,
			slippageLimit: 10000000,
		};
		let amount_sst_to_sell = parse_ether("0.01")?;
		let pending = matching_engine
			.marketSellETH(tx.quote, tx.isMaker, tx.n, tx.recipient, tx.slippageLimit)
			.value(amount_sst_to_sell)
			.send()
			.await?;
		let receipt = pending.get_receipt().await?;
		// let ret = pending.
		let url = format!(
			"https://shannon-explorer.somnia.network/tx/{}",
			receipt.transaction_hash
		);
		info!("Bought some USDC but selling some of my SST: {url}");

		let sst = ParseUnits::from(provider.get_balance(me).await?).format_units(Unit::ETHER);
		info!(sst, "My native SST balance post");

		let usdc = ERC20::new(standard_sdk::USDC, &provider);
		let name = usdc.name().call().await?;
		let my_balance = usdc.balanceOf(me).call().await?;
		info!(?name, ?my_balance, "My USDC balance post");

		let inner = receipt.inner.as_receipt().unwrap().logs.get(0).unwrap();
		let inner = &inner.inner.data;
	};

	let tx = cancelOrdersCall {
		base: todo!(),
		quote: vec![USDC],
		isBid: vec![false],
		orderIds: vec![650],
	};

	Ok(())
}
