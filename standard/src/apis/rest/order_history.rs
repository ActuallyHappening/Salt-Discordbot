use crate::{apis::rest::{orders::Order, StandardRestApi}, prelude::*};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderHistoryPage {
	pub order_histories: Vec<Order>,
	pub page_size: u16,
	pub total_count: u16,
	pub total_pages: u16,
}

impl StandardRestApi {
	/// https://learn.standardweb3.com/apps/spot/for-developers/rest-api#get-api-orderhistory-address-pagesize-page
	pub async fn get_account_order_history_page(
		&self,
		address: Address,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<OrderHistoryPage> {
		self.get([
			"api",
			"orderhistory",
			&address.to_string(),
			&page_size.to_string(),
			&page.to_string(),
		])
		.await
	}
}

#[tokio::test]
async fn standard_rest_order_history_page() {
	crate::app_tracing::install_tracing("info").ok();

	let client = StandardRestApi::default();
	let example = address!("0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0");
	let page = client
		.get_account_order_history_page(example, u16!(10), u16!(1))
		.await;

	info!(?page);
}
