mod tracing;

#[allow(unused_imports)]
use ::tracing::{debug, error, info, trace, warn};
use color_eyre::eyre::Context as _;
use salt_sdk::{GasEstimator, LiveLogging, Salt, TransactionInfo};

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

	// let amount = format!("0.00001");
	// let vault_address = format!("0x85BCADfB48E95168b3C4aA3221ca2526CF96c99E");
	// let recipient_address = format!("0xEA428233445A5Cf500B9d5c91BcA6E7B887f7D70");
	let amount = rl.readline("Amount to transfer: ")?;
	let vault_address = rl.readline("Vault address: ")?;
	let recipient_address = rl.readline("Recipient address: ")?;

	let transaction = salt
		.transaction(TransactionInfo {
			amount: &amount,
			vault_address: &vault_address,
			recipient_address: &recipient_address,
			data: "",
			logging: LiveLogging::from(|msg| info!(%msg)),
			gas: GasEstimator::Default,
		})
		.await
		.wrap_err("Couldn't do salt transaction")?;

	info!("Salt transaction completed:\n{}", transaction);

	Ok(())
}
