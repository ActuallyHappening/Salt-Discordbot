use alloy::{
	eips::Encodable2718, network::{EthereumWallet, TransactionBuilder}, primitives::{address, utils::parse_ether, Address, Bytes, U256}, providers::Provider, rpc::types::TransactionRequest, signers::local::PrivateKeySigner, sol
};
use tracing::{debug, error, info, trace, warn};

#[path = "tracing.rs"]
mod app_tracing;

const VAULT: Address = address!("0x14E559E06a524fb546e2eaceF79A11aEC85c16C2");
const SEPOLIA_ARBITRUM_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,intu_sdk=trace")?;

	let signer: PrivateKeySigner = include_str!("private_key").parse()?;
	let wallet = EthereumWallet::new(signer);
	let provider = alloy::providers::ProviderBuilder::new()
		.wallet(wallet.clone())
		.connect(SEPOLIA_ARBITRUM_RPC)
		.await?;

	let intu_contract = intu_sdk::abi::IntuVault::new(VAULT, provider);

	let vault_info = intu_contract.vaultInfos().call().await?;
	info!("Found info for vault in Rust: {:?}", vault_info);

	let transactions = intu_contract.transactions(U256::from(84)).call().await?;
	info!(?transactions);

	let me = address!("0xEA428233445A5Cf500B9d5c91BcA6E7B887f7D70");
	let amount = parse_ether("0.5");
	let tx = TransactionRequest::default()
		.with_to(me)
		.with_chain_id(421614)
		.build(&wallet)
		.await?;
	let tx = tx.encoded_2718();

	// let test_transaction = TransactionBuilder {};
	let pending = intu_contract
		.proposeTransaction(transactionInfo, String::from("first time?"))
		.send()
		.await?;

	Ok(())
}
