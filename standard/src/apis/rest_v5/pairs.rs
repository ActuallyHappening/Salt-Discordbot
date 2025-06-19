use crate::abis::orderbook::Orderbook;
use crate::apis::rest_v5::token::Token;
use crate::apis::{
	EnforceInvariants, EnforcementContext, u256_from_radix_ether, u256_from_radix_wei,
};
use crate::{apis::rest_v5::StandardRestApi_v5, prelude::*};
use alloy::network::Network;
use alloy::primitives::U256;
use alloy::providers::Provider;
use time::OffsetDateTime;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllPairsPage {
	pub pairs: Vec<Pair>,
	pub page_size: String,
	pub total_count: u32,
	pub total_pages: u32,
}

impl EnforceInvariants for AllPairsPage {
	async fn check_invariants<P, N>(&self, flags: EnforcementContext<P>) -> color_eyre::Result<()>
	where
		P: Provider<N>,
		N: Network,
		EnforcementContext<P>: Clone,
	{
		for pair in &self.pairs {
			pair.check_invariants(flags.clone()).await?;
		}

		Ok(())
	}
}

impl StandardRestApi_v5 {
	pub async fn get_all_pairs_page(
		&self,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<AllPairsPage> {
		self.get([
			"api",
			"pairs",
			&page_size.get().to_string(),
			&page.get().to_string(),
		])
		.await
	}
}

#[tokio::test]
async fn standard_rest_all_pairs_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let client = StandardRestApi_v5::default();
	let page = client.get_all_pairs_page(u16!(100), u16!(1)).await?;

	info!(?page);

	Ok(())
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
	/// Orderbook
	pub id: Address,
	/// e.g. SST/USDC
	pub symbol: String,
	/// e.g. SST/USDC
	pub ticker: String,
	/// e.g. SST/USDC
	pub description: String,
	#[serde(with = "time::serde::timestamp")]
	pub listing_date: OffsetDateTime,

	pub quote: Token,
	pub base: Token,

	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub price: U256,

	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "ath")]
	pub all_time_high: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "atl")]
	pub all_time_low: U256,
	// #[serde(deserialize_with = "u256_from_radix_wei")]
	// pub trades_count: U256,
	pub total_day_buckets: u64,
	pub total_min_buckets: u64,
	pub total_hour_buckets: u64,
	pub total_week_buckets: u64,
	pub total_month_buckets: u64,
}

impl EnforceInvariants for Pair {
	async fn check_invariants<P, N>(
		&self,
		flags: crate::apis::EnforcementContext<P>,
	) -> color_eyre::Result<()>
	where
		P: Provider<N>,
		N: Network,
	{
		let orderbook = Orderbook::new(self.id, flags.provider);
		let Orderbook::getBaseQuoteReturn { base, quote } = orderbook.getBaseQuote().call().await?;

		eyre_assert_eq!(base, self.base.id);
		eyre_assert_eq!(quote, self.quote.id);

		Ok(())
	}
}

impl StandardRestApi_v5 {
	pub async fn get_default_pair(&self) -> color_eyre::Result<Pair> {
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
