mod tracing;

#[allow(unused_imports)]
use ::tracing::{debug, error, info, trace, warn};
use color_eyre::eyre::Context as _;
use salt_sdk::Salt;

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

	let amount = rl.readline("Amount to transfer: ")?;
	let vault_address = rl.readline("Vault address: ")?;
	let recipient_address = rl.readline("Recipient address: ")?;

	salt.transaction(&amount, &vault_address, &recipient_address)
		.await
		.wrap_err("Couldn't do salt transaction")?;

	Ok(())
}
