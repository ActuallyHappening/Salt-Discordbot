mod tracing;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	tracing::install_tracing()?;

	::tracing::info!("Started logging for the discord server");

	rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.expect("Couldn't install default crypto provider");

	salt_discordbot::main().await?;

	::tracing::info!("Stopping discord server cleanly");

	Ok(())
}
