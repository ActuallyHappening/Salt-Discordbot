use alloy::primitives::{Address, Bytes};

use crate::apis::CheckInvariants;
use crate::prelude::*;
use crate::{apis::rest_v4::StandardRestApi_v4, app_tracing};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeData {
	pub id: String,
	pub network_name: String,
	pub bytecode: Bytes,
	/// OrderbookFactory
	pub deployer: Address,
	pub total_day_buckets: u64,
	pub total_week_buckets: u64,
	pub total_month_buckets: u64,
}

impl CheckInvariants for ExchangeData {
	async fn check_invariants(&self) -> color_eyre::Result<()> {
		Ok(())
	}
}

impl StandardRestApi_v4 {
	/// /api/exchange
	pub async fn get_exchange_data(&self) -> color_eyre::Result<ExchangeData> {
		self.get(["api", "exchange"]).await
	}
}

#[tokio::test]
async fn standard_rest4_exchange_data() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v4::default();
	let data = client.get_exchange_data().await?;
	info!(?data);

	Ok(())
}
