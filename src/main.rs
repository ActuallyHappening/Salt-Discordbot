
#[path = "tracing.rs"]
mod app_tracing;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	let _guard = app_tracing::install_tracing("info,salt_discord=debug,salt_sdk=debug").await?;

	::tracing::info!("Started logging for the discord server");

	rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.expect("Couldn't install default crypto provider");
	
	salt_discordbot::main().await;

	::tracing::info!("Stopping discord server cleanly");

	Ok(())
}
