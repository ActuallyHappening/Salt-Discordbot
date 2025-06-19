use alloy::network::Network;
use alloy::primitives::{Address, Bytes};
use alloy::providers::{Provider, ProviderBuilder};

use crate::abis::matching_engine::CONTRACT_ADDRESS;
use crate::abis::orderbook_factory::OrderbookFactory;
use crate::apis::{EnforceInvariants, EnforcementContext, RPC_URL};
use crate::prelude::*;
use crate::{apis::rest_v5::StandardRestApi_v5, app_tracing};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeData {
	/// e.g. standard-exchange
	pub id: String,
	pub bytecode: Bytes,
	/// OrderbookFactory
	pub deployer: Address,
	pub total_day_buckets: u64,
	pub total_week_buckets: u64,
	pub total_month_buckets: u64,
}

impl EnforceInvariants for ExchangeData {
	async fn check_invariants<P, N>(&self, flags: EnforcementContext<P>) -> color_eyre::Result<()>
	where
		P: Provider<N>,
		N: Network,
		EnforcementContext<P>: Clone,
	{
		if &self.id != "standard-exchange" {
			trace!(?self.id, "Unrecognised id");
		}

		let provider = ProviderBuilder::new().connect(RPC_URL).await?;
		let orderbook_factory = OrderbookFactory::new(self.deployer, provider);

		eyre_assert_eq!(orderbook_factory.engine().call().await?, CONTRACT_ADDRESS);

		let version = orderbook_factory.version().call().await?;
		if version != 0 {
			warn!(?version, "Unknown orderbook factory version");
		}

		Ok(())
	}
}

impl StandardRestApi_v5 {
	/// /api/exchange
	pub async fn get_exchange_data(&self) -> color_eyre::Result<ExchangeData> {
		self.get(["api", "exchange"]).await
	}
}

#[tokio::test]
async fn standard_rest_exchange_data() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v5::default();
	let data = client.get_exchange_data().await?;
	info!(?data);

	Ok(())
}
