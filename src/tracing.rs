use time::{UtcOffset, macros::format_description};
use tracing_subscriber::fmt::time::OffsetTime;

pub fn install_tracing(filter: &str) -> color_eyre::Result<()> {
	use tracing_error::ErrorLayer;
	use tracing_subscriber::prelude::*;
	use tracing_subscriber::{EnvFilter, fmt};

	let offset = UtcOffset::current_local_offset().unwrap_or_else(|err| {
		::tracing::warn!(message = "Couldn't find local time offset", ?err);
		UtcOffset::UTC
	});
	let timer = OffsetTime::new(offset, format_description!("[hour]:[minute]:[second]"));

	let fmt_layer = fmt::layer().with_target(true).with_timer(timer);
	let filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(filter))
		.unwrap();

	tracing_subscriber::registry()
		.with(filter_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();

	color_eyre::install()?;

	Ok(())
}
