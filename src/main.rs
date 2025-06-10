mod tracing;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	tracing::install_tracing("info,salt_discord=trace,salt_sdk=debug")?;

	::tracing::info!("Started logging for the discord server");

	rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.expect("Couldn't install default crypto provider");

	loop {
		match salt_discordbot::main().await {
			Ok(()) => {
				// ctrlc, clean exit, actually exit
				break;
			}
			Err(err) => {
				::tracing::error!(?err, "Top level error!");
				// keep looping
			}
		}
		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
	}

	::tracing::info!("Stopping discord server cleanly");

	Ok(())
}
