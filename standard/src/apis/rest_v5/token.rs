use std::{error::Error, marker::PhantomData, str::FromStr};

use alloy::primitives::U256;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{apis::rest_v5::StandardRestApi_v5, prelude::*};
use crate::apis::{u256_from_radix_ether, lazy_empty_str};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct OuterTokenData {
	pub token: Token,
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
	/// ERC20 address
	pub id: Address,
	pub name: String,
	pub symbol: String,
	#[serde(deserialize_with = "u256_from_radix_ether")]
	pub total_supply: U256,
	#[serde(rename = "logoURI")]
	pub logo_uri: Url,
	pub decimals: u8,
	#[serde(rename = "priceUSD")]
	pub price_usd: f64,
	pub listing_date: u128,
	#[serde(rename = "dayVolumeUSD")]
	pub day_volume_usd: f64,
	#[serde(deserialize_with = "lazy_empty_str")]
	pub creator: Option<Address>,
	pub total_min_buckets: u64,
	pub total_hour_buckets: u64,
	pub total_day_buckets: u64,
	pub total_week_buckets: u64,
	pub total_month_buckets: u64,
	#[serde(default)]
	pub tags: Vec<String>,
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
impl StandardRestApi_v5 {
	pub async fn get_token_data(&self, address: Address) -> color_eyre::Result<OuterTokenData> {
		self.get(["api", "token", &address.to_string()]).await
	}
}

#[tokio::test]
async fn standard_rest_get_token_data() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v5::default();
	let examples = [crate::USDC, crate::WBTC, crate::WSOL];
	for example in examples {
		let token_data = client.get_token_data(example).await?;
		info!(?token_data);
	}

	Ok(())
}