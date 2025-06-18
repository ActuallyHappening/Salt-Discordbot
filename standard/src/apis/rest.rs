#![allow(unused)]
//! https://learn.standardweb3.com/apps/spot/for-developers/rest-api

use crate::{app_tracing, prelude::*};

use alloy::primitives::{ruint::aliases::U256, utils::parse_ether};
use color_eyre::Section;
use serde::de::DeserializeOwned;
use std::borrow::Borrow;
use url::Url;

pub struct StandardRestApi {
	base: Url,
	client: reqwest::Client,
}

impl Default for StandardRestApi {
	fn default() -> Self {
		StandardRestApi {
			base: "https://somnia-testnet-ponder-v5.standardweb3.com/"
				.parse()
				.unwrap(),
			client: reqwest::Client::new(),
		}
	}
}

impl StandardRestApi {
	pub async fn get<T>(
		&self,
		path: impl IntoIterator<Item = impl Borrow<str>>,
	) -> color_eyre::Result<T>
	where
		T: DeserializeOwned,
	{
		let mut url = self.base.clone();
		let mut url_path = url
			.path_segments_mut()
			.map_err(|_| eyre!("Couldn't access path of base URL"))?;
		for segment in path {
			url_path.push(segment.borrow());
		}
		drop(url_path);
		let resp = self
			.client
			.get(url)
			.send()
			.await
			.wrap_err("Couldn't get {url}")?;
		let str = resp.text().await.wrap_err("Body not text")?;
		let json: serde_json::Value = serde_json::from_str(&str)?;
		let str = serde_json::to_string_pretty(&json)?;
		{
			// write to data.json
			let path = "/home/ah/Desktop/Salt-Discordbot/standard/src/data.json";
			std::fs::write(path, &str).unwrap();
		}
		serde_json::from_str(&str)
			.wrap_err("Couldn't deserialize a get request")
			// .note(format!("Original response: {str}"))
	}
}

pub mod exchange;
pub mod orders;
pub mod token;
pub mod trade_history;

/// From a base 10 string encoding of a large number
pub(crate) fn u256_from_radix_wei<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: serde::Deserializer<'de>,
{
	// serde_json -F arbitrary-precision
	use serde::Deserialize as _;
	let s = serde_json::Number::deserialize(deserializer)?;
	let s = s.to_string();
	if s.contains("e+") {
		// handle 1e+22 case
		let exp_str = s
			.split("e+")
			.nth(1)
			.ok_or(serde::de::Error::custom("expected something after 'e'"))?;
		let exp: u8 = exp_str.parse().map_err(serde::de::Error::custom)?;
		let exp: U256 = exp.try_into().map_err(serde::de::Error::custom)?;
		let mantissa = exp_str
			.split("e+")
			.nth(0)
			.ok_or(serde::de::Error::custom("expected orig"))?;
		let num = U256::from_str_radix(&mantissa, 10).map_err(serde::de::Error::custom)?;
		Ok(num.pow(exp))
	} else {
		let num = U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
		Ok(num)
	}
}

/// From a base 10 string encoding of a large number
pub(crate) fn u256_from_radix_ether<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: serde::Deserializer<'de>,
{
	// serde_json -F arbitrary-precision
	use serde::Deserialize as _;
	let s = serde_json::Number::deserialize(deserializer)?;
	let s = s.to_string();
	if s.contains("e+") {
		// handle 1e+22 case
		let exp_str = s
			.split("e+")
			.nth(1)
			.ok_or(serde::de::Error::custom("expected something after 'e'"))?;
		let exp: u8 = exp_str.parse().map_err(serde::de::Error::custom)?;
		let exp: U256 = exp.try_into().map_err(serde::de::Error::custom)?;
		let mantissa = exp_str
			.split("e+")
			.nth(0)
			.ok_or(serde::de::Error::custom("expected orig"))?;
		let num = U256::from_str_radix(&mantissa, 10).map_err(serde::de::Error::custom)?;
		Ok(num.pow(exp))
	} else {
		// let num = U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
		let num = parse_ether(&s).map_err(serde::de::Error::custom)?;
		Ok(num)
	}
}
