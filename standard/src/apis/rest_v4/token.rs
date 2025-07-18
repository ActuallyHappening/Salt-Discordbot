use std::{error::Error, marker::PhantomData, str::FromStr};

use alloy::primitives::U256;
use alloy::providers::ProviderBuilder;
use serde::Deserialize;
use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::apis::{lazy_empty_str, u256_from_radix_ether, u256_from_radix_wei, CheckInvariants, RPC_URL};
use crate::{apis::rest_v4::StandardRestApi_v4, prelude::*};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OuterTokenData {
	pub token: Token,
	pub base_pairs: Vec<serde_json::Value>,
	pub latest_day_bucket: Option<Bucket>,
	pub latest_hour_bucket: Option<Bucket>,
	pub latest_min_bucket: Option<Bucket>,
	pub latest_month_bucket: Option<Bucket>,
	pub latest_week_bucket: Option<Bucket>,
}

impl CheckInvariants for OuterTokenData {
	async fn check_invariants(&self) -> color_eyre::Result<()> {
		self.token.check_invariants().await?;
		Ok(())
	}
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub average: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub close: U256,
	pub count: u64,
	/// idk
	pub difference: serde_json::Number,
	/// idk
	pub difference_percentage: serde_json::Number,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub high: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub low: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub open: U256,
	/// idk
	pub symbol: Option<serde_json::Value>,
	#[serde(with = "time::serde::timestamp")]
	pub timestamp: OffsetDateTime,
	/// ERC20
	pub token: Address,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "tvl")]
	pub total_volume: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "tvlUSD")]
	pub total_volume_usd: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub volume: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "volumeUSD")]
	pub volume_usd: U256,
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
	/// ERC20 address
	pub id: Address,
	/// why is this sometimes an empty string?
	#[serde(deserialize_with = "lazy_empty_str")]
	pub creator: Option<Address>,
	pub name: String,
	pub symbol: String,
	/// e.g. SOL
	pub ticker: String,
	pub decimals: u8,
	#[serde(rename = "logoURI")]
	pub logo_uri: Url,
	#[serde(default)]
	pub tags: Vec<String>,
	#[serde(with = "time::serde::timestamp")]
	pub listing_date: OffsetDateTime,

	/// idk
	pub cg_id: String,
	/// idk
	pub cmc_id: String,
	/// idk
	pub cp_price: serde_json::Number,

	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "ath")]
	pub all_time_high: U256,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	#[serde(rename = "atl")]
	pub all_time_low: U256,

	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub total_supply: U256,
	#[serde(deserialize_with = "u256_from_radix_wei")]
	pub trades_count: U256,

	#[serde(rename = "priceUSD")]
	pub price_usd: f64,

	pub total_day_buckets: u64,
	pub total_min_buckets: u64,
	pub total_hour_buckets: u64,
	pub total_week_buckets: u64,
	pub total_month_buckets: u64,
}

impl CheckInvariants for Token {
	async fn check_invariants(&self) -> color_eyre::Result<()> {
		let provider = ProviderBuilder::new().connect(RPC_URL).await?;
		Ok(())
	}
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
impl StandardRestApi_v4 {
	pub async fn get_token_data(&self, address: Address) -> color_eyre::Result<OuterTokenData> {
		self.get(["api", "token", &address.to_string()]).await
	}
}

#[tokio::test]
async fn standard_rest4_get_token_data() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v4::default();
	let examples = [crate::USDC, crate::WBTC, crate::WSOL];
	for example in examples {
		let token_data = client.get_token_data(example).await?;
		info!(?token_data);
	}

	Ok(())
}
