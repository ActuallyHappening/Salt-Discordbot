use alloy::{
	primitives::utils::{ParseUnits, Unit, parse_ether},
	providers::{Provider, ProviderBuilder},
	signers::local::PrivateKeySigner,
};
use clap::Parser;
use standard_sdk::{
	USDC,
	abis::{
		matching_engine::{
			self,
			MatchingEngine::{self, cancelOrdersCall, marketBuyETHCall, marketSellETHCall},
		},
		orderbook::Orderbook,
		orderbook_factory::OrderbookFactory,
	},
	apis::rest::StandardRestApi,
	prelude::*,
};
use time::UtcOffset;
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

#[derive(clap::Parser, Debug)]
enum Cli {
	Read,
	SellEth,
	BuyEth,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,standard=trace")?;
	trace!("Started tracing");

	let cli = Cli::parse();

	let private_key = include_str!("private-key");
	let signer: PrivateKeySigner = private_key.parse()?;
	let me = signer.address();
	let provider = ProviderBuilder::new()
		.wallet(signer.clone())
		.connect("https://dream-rpc.somnia.network/")
		.await?;
	let rest_api = StandardRestApi::default();

	{
		let sst = ParseUnits::from(provider.get_balance(me).await?).format_units(Unit::ETHER);
		info!(%sst, "My native SST balance pre");
	}
	{
		let usdc = ERC20::new(standard_sdk::USDC, &provider);
		let name = usdc.name().call().await?;
		let decimals = usdc.decimals().call().await?;
		let my_balance =
			ParseUnits::from(usdc.balanceOf(me).call().await?).format_units(decimals.try_into()?);
		info!(%name, %my_balance, "My USDC balance pre");
	}
	{
		// order history using smart contract
		let matching_engine = MatchingEngine::new(matching_engine::CONTRACT_ADDRESS, &provider);

		let orderbook_factory_addr = matching_engine.orderbookFactory().call().await?;
		info!(?orderbook_factory_addr);

		let orderbook_factory = OrderbookFactory::new(orderbook_factory_addr, &provider);
		let engine_addr = orderbook_factory.engine().call().await?;
		info!(?engine_addr);

		let pair = standard_sdk::abis::orderbook_factory::IOrderbookFactory::Pair {
			base: address!("0x4a3bc48c156384f9564fd65a53a2f3d534d8f2b7"),
			quote: address!("0x0ed782b8079529f7385c3eda9faf1eaa0dbc6a17"),
		};
		let orderbook_addr = orderbook_factory
			.getPair(pair.base, pair.quote)
			.call()
			.await?;
		info!(?orderbook_addr);

		let orderbook = Orderbook::new(orderbook_addr, &provider);
		let last_market_price = orderbook.lmp().call().await?;
		info!(?last_market_price);
	}
	{
		// order history using REST API
		let mut history = vec![];
		history.extend(
			rest_api
				.get_account_trade_history_page(me, u16!(3), u16!(1))
				.await?
				.trade_histories,
		);
		history.extend(
			rest_api
				.get_account_trade_history_page(me, u16!(3), u16!(2))
				.await?
				.trade_histories,
		);
		let time_formatter = time::macros::format_description!(
			"[day]/[month]/[year] [hour]:[minute]:[second] +[offset_hour]"
		);
		let offset = UtcOffset::current_local_offset().unwrap_or_else(|err| {
			::tracing::warn!(message = "Couldn't find local time offset", ?err);
			UtcOffset::UTC
		});
		for order in history {
			let time = order.timestamp.to_offset(offset).format(&time_formatter)?;

			let base_asset = &order.base.name;
			let base_amount =
				ParseUnits::from(order.amount).format_units(order.base.decimals.try_into()?);

			let quote_asset = &order.quote.name;

			let price = ParseUnits::from(order.price).format_units(Unit::ETHER);

			debug!(
				"{time} | Sold {base_amount} {base_asset} for {quote_asset} | Priced at {price}",
			);
			// trace!("{:#?}", order);
			// let base = &order.base.addr;
			// let quote = &order.quote.addr;
			// trace!(?base, ?quote);
		}
	}

	let matching_engine = standard_sdk::abis::matching_engine::MatchingEngine::new(
		standard_sdk::abis::matching_engine::CONTRACT_ADDRESS,
		&provider,
	);
	if matches!(cli, Cli::SellEth) {
		let tx = marketSellETHCall {
			quote: USDC,
			isMaker: true,
			n: 5,
			recipient: me,
			slippageLimit: 10u32.pow(5),
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
		let decimals = usdc.decimals().call().await?;
		let my_balance =
			ParseUnits::from(usdc.balanceOf(me).call().await?).format_units(decimals.try_into()?);
		info!(%name, %my_balance, "My USDC balance pre");
	};
	if matches!(cli, Cli::BuyEth) {
		let tx = marketBuyETHCall {
			base: USDC,
			isMaker: true,
			n: 5,
			recipient: me,
			slippageLimit: 10u32.pow(5),
		};
		let usdc_decimals: Unit = ERC20::new(USDC, provider)
			.decimals()
			.call()
			.await?
			.try_into()?;
		let amount_usdc_to_sell = ParseUnits::parse_units("0.01", usdc_decimals)?.get_absolute();
	}

	let orders_page = rest_api.get_orders_page(me, u16!(10), u16!(1)).await?;
	info!(?orders_page);

	// let tx = cancelOrdersCall {
	// 	base: todo!(),
	// 	quote: vec![USDC],
	// 	isBid: vec![false],
	// 	orderIds: vec![650],
	// };

	Ok(())
}
