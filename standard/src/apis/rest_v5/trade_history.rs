use crate::{
	apis::{
		EnforceInvariants,
		rest_v5::{StandardRestApi_v5, orders::Order},
	},
	prelude::*,
};

/// paginated
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradeHistoryPage {
	pub trade_histories: Vec<Order>,
	pub total_count: u16,
	pub total_pages: u16,
	pub page_size: u16,
}

impl EnforceInvariants for TradeHistoryPage {
	async fn check_invariants<P, N>(
		&self,
		flags: crate::apis::EnforcementContext<P>,
	) -> color_eyre::Result<()>
	where
		P: alloy::providers::Provider<N>,
		N: alloy::network::Network,
		crate::apis::EnforcementContext<P>: Clone,
	{
		let flags = flags.expect_historical_orders();
		for trade in &self.trade_histories {
			trade.check_invariants(flags.clone()).await?;
		}
		Ok(())
	}
}

impl StandardRestApi_v5 {
	pub async fn get_account_trade_history_page(
		&self,
		address: Address,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<TradeHistoryPage> {
		self.get([
			"api",
			"tradehistory",
			&address.to_string(),
			&page_size.to_string(),
			&page.to_string(),
		])
		.await
	}
}

#[tokio::test]
async fn standard_account_trade_history_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi_v5::default();
	let example = address!("0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0");
	let trade_history_page = client
		.get_account_trade_history_page(example, u16!(10), u16!(1))
		.await?;
	info!(?trade_history_page);

	Ok(())
}
