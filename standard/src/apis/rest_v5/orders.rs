use std::num::NonZero;

use alloy::{
	network::Network,
	primitives::{TxHash, U256},
	providers::{Provider, ProviderBuilder},
};
use time::OffsetDateTime;

use crate::{
	abis::orderbook::{ExchangeOrderbook, Orderbook},
	apis::{
		EnforceInvariants, EnforcementContext, RPC_URL,
		rest_v5::{StandardRestApi_v5, token::Token},
		u256_from_radix_ether,
	},
	prelude::*,
};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
	/// owner
	pub account: Address,
	pub tx_hash: TxHash,
	#[serde(with = "time::serde::timestamp")]
	pub timestamp: OffsetDateTime,
	pub order_id: u32,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub amount: U256,

	pub base: Token,
	pub quote: Token,

	// #[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(default)]
	pub placed: Option<serde_json::Number>,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub price: U256,

	/// Orderbook
	pub pair: Address,
	/// e.g. SST/USDC
	pub pair_symbol: String,

	pub is_bid: bool,
	/// Is true when looking at order history
	#[serde(default)]
	pub closed: bool,
}

impl EnforceInvariants for Order {
	async fn check_invariants<P, N>(&self, flags: EnforcementContext<P>) -> color_eyre::Result<()>
	where
		P: Provider<N>,
		N: Network,
		EnforcementContext<P>: Clone,
	{
		let orderbook = Orderbook::new(self.pair, &flags.provider);

		// orderIDs are re-used, so fetching from blockchain will give random stuff

		let Orderbook::getBaseQuoteReturn { base, quote } = orderbook.getBaseQuote().call().await?;

		// eyre_assert_eq!(base, self.base.id);
		// eyre_assert_eq!(quote, self.quote.id);

		self.base.check_invariants(flags.clone()).await?;
		self.quote.check_invariants(flags.clone()).await?;

		Ok(())
	}
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OuterOrdersPage {
	pub orders: Vec<Order>,
	pub total_count: u16,
	pub total_pages: u16,
	pub page_size: NonZero<usize>,
}

impl EnforceInvariants for OuterOrdersPage {
	async fn check_invariants<P, N>(&self, flags: EnforcementContext<P>) -> color_eyre::Result<()>
	where
		P: Provider<N>,
		N: Network,
		EnforcementContext<P>: Clone,
	{
		for order in &self.orders {
			order.check_invariants(flags.clone()).await?;
		}
		Ok(())
	}
}

impl StandardRestApi_v5 {
	pub async fn get_orders_page(
		&self,
		address: Address,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<OuterOrdersPage> {
		self.get([
			"api",
			"orders",
			&address.to_string(),
			&page_size.to_string(),
			&page.to_string(),
		])
		.await
	}
}

#[tokio::test]
async fn standard_rest_get_orders_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v5::default();
	let example_address = address!("0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0");
	let page = client
		.get_orders_page(example_address, u16!(10), u16!(1))
		.await?;
	info!(?page);
	Ok(())
}

// #[tokio::test]
// async fn testing() -> color_eyre::Result<()> {
// 	crate::app_tracing::install_test_tracing();

// 	let path = "/home/ah/Desktop/Salt-Discordbot/standard/src/data.json";
// 	let str = std::fs::read_to_string(path)?;

// 	let data: OuterOrdersPage = serde_json::from_str(&str)?;

// 	data.check_invariants().await?;

// 	Ok(())
// }
