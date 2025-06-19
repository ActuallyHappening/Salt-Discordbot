use crate::apis::rest_v5::token::Token;
use crate::apis::u256_from_radix_ether;
use crate::{apis::rest_v5::StandardRestApi_v5, prelude::*};
use alloy::primitives::U256;
use time::OffsetDateTime;

impl StandardRestApi_v5 {
	pub async fn get_all_pairs_page(
		&self,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<serde_json::Value> {
		self.get(["api", "pairs", &page.to_string(), &page_size.to_string()])
			.await
	}
}

#[tokio::test]
async fn standard_rest_all_pairs_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let client = StandardRestApi_v5::default();
	let page = client.get_all_pairs_page(u16!(10), u16!(1)).await?;

	info!(?page);

	Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairData {
	// #[serde(deserialize_with = "u256_from_radix_ether")]
	pub ath: f64,
	pub atl: f64,
	pub b_decimal: u8,
	pub base: Token,
	/// Orderbook
	pub id: Address,
	pub quote: Token,
	/// e.g. SST/USDC
	pub description: String,
	/// e.g. SST/USDC
	pub symbol: String,
	/// e.g. SST/USDC
	pub ticker: String,
	#[serde(rename = "type")]
	pub market_type: String,
	/// idk
	pub trades_count: Option<serde_json::Value>,
	/// e.g. Standard
	pub exchange: String,
	#[serde(with = "time::serde::timestamp")]
	pub listing_date: OffsetDateTime,
}

impl StandardRestApi_v5 {
	pub async fn get_default_pair(&self) -> color_eyre::Result<PairData> {
		self.get(["api", "pair", "default"]).await
	}
}

#[tokio::test]
async fn standard_rest_default_pair() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let client = StandardRestApi_v5::default();
	let pair = client.get_default_pair().await?;

	info!(?pair);

	Ok(())
}
