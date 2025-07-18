use time::{UtcOffset, macros::format_description};
use tracing_subscriber::fmt::time::OffsetTime;

#[allow(unused)]
pub fn install_tracing(
	default_env_filter: impl std::borrow::Borrow<str>,
) -> color_eyre::Result<()> {
	use tracing_error::ErrorLayer;
	use tracing_subscriber::prelude::*;
	use tracing_subscriber::{EnvFilter, fmt};

	let offset = UtcOffset::current_local_offset().unwrap_or_else(|err| {
		::tracing::warn!(message = "Couldn't find local time offset", ?err);
		UtcOffset::UTC
	});
	let timer = OffsetTime::new(
		offset,
		format_description!("[hour]:[minute]:[second] +[offset_hour]"),
	);

	let fmt_layer = fmt::layer().with_target(true).with_timer(timer);
	let filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new(default_env_filter.borrow()))
		.unwrap();

	tracing_subscriber::registry()
		.with(filter_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.try_init()?;

	color_eyre::install()?;

	Ok(())
}

#[cfg(test)]
#[allow(unused)]
pub fn install_test_tracing() {
	install_tracing("info").ok();
}
