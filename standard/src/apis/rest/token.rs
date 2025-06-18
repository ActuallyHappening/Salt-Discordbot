use std::{error::Error, marker::PhantomData, str::FromStr};

use alloy::primitives::U256;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{apis::rest::StandardRestApi, prelude::*};
use super::u256_from_radix;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TokenData {
	pub token: InnerTokenData,
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InnerTokenData {
	pub id: String,
	pub name: String,
	pub symbol: String,
	#[serde(deserialize_with = "u256_from_radix")]
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
	pub tags: Vec<String>,
}

/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-token-address
impl StandardRestApi {
	pub async fn get_token_data(&self, address: Address) -> color_eyre::Result<TokenData> {
		self.get(["api", "token", &address.to_string()]).await
	}
}

#[tokio::test]
async fn standard_rest_get_token_data() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi::default();
	let examples = [crate::USDC, crate::WBTC, crate::WSOL];
	for example in examples {
		let token_data = client.get_token_data(example).await?;
		info!(?token_data);
	}

	Ok(())
}

pub use string_or_none::lazy_empty_str;
mod string_or_none {
	use std::{marker::PhantomData, str::FromStr};

	use serde::Deserialize as _;

	pub fn lazy_empty_str<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
	where
		D: serde::Deserializer<'de>,
		T: FromStr,
		<T as FromStr>::Err: std::fmt::Display,
	{
		deserializer.deserialize_any(StringOrNone(PhantomData))
	}

	#[test]
	fn string_or_none() -> color_eyre::Result<()> {
		crate::app_tracing::install_tracing("info").ok();

		let json = serde_json::json!({ "example": null });
		#[derive(serde::Deserialize)]
		struct Example {
			#[serde(deserialize_with = "lazy_empty_str")]
			example: Option<String>,
		}
		let data: Example = serde_json::from_value(json)?;
		Ok(())
	}

	#[derive(Default)]
	struct StringOrNone<T>(PhantomData<fn() -> T>);

	impl<'de, T> serde::de::Visitor<'de> for StringOrNone<T>
	where
		T: FromStr,
		<T as FromStr>::Err: std::fmt::Display,
	{
		type Value = Option<T>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str(&format!(
				"a string ({}) or null/undefined",
				std::any::type_name::<T>()
			))
		}

		fn visit_none<E>(self) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(None)
		}

		fn visit_unit<E>(self) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(None)
		}

		fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			let s = String::deserialize(deserializer)?;
			if s.is_empty() {
				Ok(None)
			} else {
				Ok(Some(s.parse().map_err(serde::de::Error::custom)?))
			}
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			if v.is_empty() {
				Ok(None)
			} else {
				Ok(Some(v.parse().map_err(serde::de::Error::custom)?))
			}
		}
	}
}
