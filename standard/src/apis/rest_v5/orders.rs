use std::num::NonZero;

use alloy::{
	primitives::{TxHash, U256},
	providers::ProviderBuilder,
};
use time::OffsetDateTime;

use crate::{
	abis::orderbook::{ExchangeOrderbook, Orderbook},
	apis::{
		EnforceInvariants, RPC_URL,
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

	pub base: Token,
	pub quote: Token,

	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub placed: U256,
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
	async fn check_invariants(&self) -> color_eyre::Result<()> {
		let provider = ProviderBuilder::new().connect(RPC_URL).await?;
		let orderbook = Orderbook::new(self.pair, provider);
		let ExchangeOrderbook::Order {
			owner,
			price,
			depositAmount,
		} = orderbook
			.getOrder(self.is_bid, self.order_id)
			.call()
			.await?;

		// let lmp = orderbook.lmp().call().await?;
		// let ask_head = orderbook.askHead().call().await?;
		// let bid_head = orderbook.bidHead().call().await?;

		// warn!(?owner, ?price, ?depositAmount, ?self.pair, ?lmp, ?ask_head, ?bid_head);

		if owner == Address::ZERO {
			warn!(?owner, ?self.account, "Noticing that the order with id {} isn't still live on the blockchain", self.order_id);
		} else {
			eyre_assert_eq!(owner, self.account);
			eyre_assert_eq!(price, self.price);
		}

		let Orderbook::getBaseQuoteReturn { base, quote } = orderbook.getBaseQuote().call().await?;

		// eyre_assert_eq!(base, self.base.id);
		// eyre_assert_eq!(quote, self.quote.id);

		self.base.check_invariants().await?;
		self.quote.check_invariants().await?;

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
	async fn check_invariants(&self) -> color_eyre::Result<()> {
		for order in &self.orders {
			order.check_invariants().await?;
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

#[tokio::test]
async fn testing() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let path = "/home/ah/Desktop/Salt-Discordbot/standard/src/data.json";
	let str = std::fs::read_to_string(path)?;

	let data: OuterOrdersPage = serde_json::from_str(&str)?;

	data.check_invariants().await?;

	Ok(())
}
