use alloy::primitives::Address;
use url::Url;

use crate::prelude::*;

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Env {
	pub private_key: String,

	pub bot_application_id: String,
	pub bot_token: String,

	pub somnia_shannon_rpc_endpoint: Url,
	pub sepolia_arbitrum_rpc_endpoint: Url,
	pub sepolia_ethereum_rpc_endpoint: Url,
	pub polygon_amoy_rpc_endpoint: Url,
	pub faucet_testnet_salt_account_address: Address,
}

/// Only statically includes toml if building for release,
/// for better error messages
/// Can refactor this if wanted
#[allow(dead_code)]
impl Env {
	pub async fn get() -> Result<Env> {
		#[cfg(debug_assertions)]
		return Self::from_local_dev_env().await;
		#[cfg(not(debug_assertions))]
		Self::from_statically_included()
	}

	async fn from_local_dev_env() -> Result<Env> {
		let path = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dev.env.toml");
		let file = ystd::fs::read_to_string(path)
			.await
			.wrap_err("Couldn't read env.toml")?;
		let env: Env =
			toml::from_str(&file).wrap_err("env.toml not valid toml or missing required key")?;
		Ok(env)
	}

	#[cfg(not(debug_assertions))]
	fn from_statically_included() -> Result<Env> {
		static_toml::static_toml!(
			static ENV = include_toml!("env.toml");
		);
		Ok(Env {
			bot_application_id: ENV.bot_application_id.into(),
			bot_token: ENV.bot_token.into(),
			somnia_shannon_rpc_endpoint: ENV.somnia_shannon_rpc_endpoint.parse()?,
			sepolia_arbitrum_rpc_endpoint: ENV.sepolia_arbitrum_rpc_endpoint.parse()?,
			sepolia_ethereum_rpc_endpoint: ENV.sepolia_ethereum_rpc_endpoint.parse()?,
			polygon_amoy_rpc_endpoint: ENV.polygon_amoy_rpc_endpoint.parse()?,
			faucet_testnet_salt_account_address: ENV.faucet_testnet_salt_account_address.parse()?,
			private_key: ENV.private_key.into(),
		})
	}
}
