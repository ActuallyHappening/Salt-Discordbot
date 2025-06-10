use alloy::{
	primitives::{Address, address},
	providers::Provider,
	signers::local::PrivateKeySigner,
	sol,
};
use tracing::{debug, error, info, trace, warn};

#[path = "tracing.rs"]
mod app_tracing;

const VAULT: Address = address!("0x14E559E06a524fb546e2eaceF79A11aEC85c16C2");
const SEPOLIA_ARBITRUM_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,intu_sdk=trace")?;

	// let signer: PrivateKeySigner = include_str!("private_key").parse()?;
	let provider = alloy::providers::ProviderBuilder::new()
		.connect(SEPOLIA_ARBITRUM_RPC)
		.await?;

	let intu_contract = intu_sdk::IntuVault::new(VAULT, provider);

	let info = intu_contract.vaultInfos().call().await?;
	info!("Found info for vault in Rust: {:?}", info);

	Ok(())
}
