use std::num::NonZero;

use alloy::primitives::{TxHash, U256};
use time::OffsetDateTime;

use super::u256_from_radix_ether;
use crate::{
	apis::rest::{StandardRestApi, token::InnerTokenData},
	prelude::*,
};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
	pub is_bid: bool,
	pub order_id: u128,
	pub base: InnerTokenData,
	pub quote: InnerTokenData,
	pub base_symbol: String,
	pub quote_symbol: String,
	pub pair: Address,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub price: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub amount: U256,
	
	/// Only appears null in histories, maybe this is a timestamp?
	#[serde(default)]
	// #[serde(deserialize_with = "u256_from_radix_ether")]
	pub placed: Option<String>,
	
	#[serde(with = "time::serde::timestamp")]
	pub timestamp: OffsetDateTime,
	pub account: Address,
	pub tx_hash: TxHash,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetOrdersPage {
	pub orders: Vec<Order>,
	pub total_count: u16,
	pub total_pages: u16,
	pub page_size: u16,
}

impl StandardRestApi {
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
async fn standard_rest_get_orders_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi::default();
	let example_address = address!("0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0");
	let page = client
		.get_orders_page(example_address, u16!(10), u16!(1))
		.await?;
	info!(?page);
	Ok(())
}
