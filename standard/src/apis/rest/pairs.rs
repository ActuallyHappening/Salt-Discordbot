use crate::{apis::rest::StandardRestApi, prelude::*};

impl StandardRestApi {
	pub async fn get_all_pairs_page(
		&self,
		page_size: NonZero<u16>,
		page: NonZero<u16>,
	) -> color_eyre::Result<serde_json::Value> {
		self.get(["api", "pairs", &page.to_string(), &page_size.to_string()])
			.await
	}
}

#[tokio::test]
async fn standard_rest_all_pairs_page() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let client = StandardRestApi::default();
	let page = client.get_all_pairs_page(u16!(10), u16!(1)).await?;

	info!(?page);

	Ok(())
}

impl StandardRestApi {
	pub async fn get_default_pair(&self) -> color_eyre::Result<serde_json::Value> {
		self.get(["api", "pair", "default"]).await
	}
}

#[tokio::test]
async fn standard_rest_default_pair() -> color_eyre::Result<()> {
	crate::app_tracing::install_test_tracing();

	let client = StandardRestApi::default();
	let pair = client.get_default_pair().await?;

	info!(?pair);

	Ok(())
}
