use std::num::NonZero;

use alloy::primitives::{TxHash, U256};
use time::OffsetDateTime;

use crate::{
	apis::rest_v4::{StandardRestApi_v4, token::Token},
	apis::u256_from_radix_ether,
	prelude::*,
};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
	pub account: Address,
	pub tx_hash: TxHash,
	#[serde(with = "time::serde::timestamp")]
	pub timestamp: OffsetDateTime,

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
	pub order_id: u128,
	// /// Is true when looking at order history
	// #[serde(default)]
	// pub closed: bool,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersPage {
	pub orders: Vec<Order>,
	pub total_count: u16,
	pub total_pages: u16,
	pub page_size: u16,
}

impl StandardRestApi_v4 {
	pub async fn get_orders_page(
		&self,
		address: Address,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<GetOrdersPage> {
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
async fn standard_rest4_get_orders_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v4::default();
	let example_address = address!("0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0");
	let page = client
		.get_orders_page(example_address, u16!(10), u16!(1))
		.await?;
	info!(?page);
	Ok(())
}
