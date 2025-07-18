mod tracing;

#[allow(unused_imports)]
use ::tracing::{debug, error, info, trace, warn};
use alloy_primitives::{address, utils::parse_ether};
use color_eyre::eyre::Context as _;
use salt_sdk::{GasEstimator, LiveLogging, Salt, TransactionInfo};
use std::time::Duration;
use ystd::prelude::*;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	tracing::install_tracing("info,salt_sdk=trace,salt_cli=trace")?;

	trace!("Started salt-sdk tracing");

	std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"))?;
	let env_file = std::fs::read_to_string("env.toml")
		.wrap_err("Couldn't fine env.toml next to Cargo.toml")?;
	let config: salt_sdk::SaltConfig = toml::from_str(&env_file)
		.wrap_err("env.toml isn't valid toml format or missing required keys")?;

	let salt = Salt::new(config)?;

	let mut rl = rustyline::DefaultEditor::new()?;

	let amount = format!("0.00001");
	let vault_address = address!("0x85BCADfB48E95168b3C4aA3221ca2526CF96c99E");
	let recipient_address = address!("0xEA428233445A5Cf500B9d5c91BcA6E7B887f7D70");
	// let amount = rl.readline("Amount to transfer: ")?;
	// let vault_address = rl.readline("Vault address: ")?.parse()?;
	// let recipient_address = rl.readline("Recipient address: ")?.parse()?;

	let amount = parse_ether(&amount)?;

	salt.transaction(TransactionInfo {
		amount,
		vault_address,
		recipient_address,
		data: vec![],
		logging: &mut LiveLogging::from_cb(|msg| info!(%msg)),
		gas: GasEstimator::Default,
		confirm_broadcast: true,
		auto_broadcast: true,
	})
	// .timeout(Duration::from_secs(60 * 2))
	.await
	// .wrap_err("Transaction timed out")?
	.wrap_err("Couldn't do salt transaction")?;

	info!("Salt transaction completed");

	Ok(())
}
