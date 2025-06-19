#![allow(unused)]
//! https://learn.standardweb3.com/apps/spot/for-developers/rest-api

use crate::{apis::{EnforceInvariants, EnforcementFlags}, app_tracing, prelude::*};

use alloy::primitives::{ruint::aliases::U256, utils::parse_ether};
use color_eyre::Section;
use serde::de::DeserializeOwned;
use std::borrow::Borrow;
use url::Url;

#[allow(non_camel_case_types)]
pub struct StandardRestApi_v5 {
	base: Url,
	client: reqwest::Client,
}

impl Default for StandardRestApi_v5 {
	fn default() -> Self {
		StandardRestApi_v5 {
			base: "https://somnia-testnet-ponder-v5.standardweb3.com/"
				.parse()
				.unwrap(),
			client: reqwest::Client::new(),
		}
	}
}

impl StandardRestApi_v5 {
	pub async fn get<T>(
		&self,
		path: impl IntoIterator<Item = impl Borrow<str>>,
	) -> color_eyre::Result<T>
	where
		T: DeserializeOwned + EnforceInvariants,
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
		let data: T = serde_json::from_str(&str).wrap_err("Couldn't deserialize a get request")?;

		data.check_invariants(EnforcementFlags::default()).await?;

		Ok(data)
	}
}

pub mod exchange;
pub mod order_history;
pub mod orders;
// pub mod pairs;
pub mod token;
// pub mod trade_history;
