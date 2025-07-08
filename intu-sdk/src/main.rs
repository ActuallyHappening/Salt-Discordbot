use std::str::FromStr as _;

use alloy::{
	consensus::{
		EthereumTxEnvelope, EthereumTypedTransaction, SignableTransaction, Transaction, TxEip4844,
		TxLegacy,
	},
	eips::{BlockNumberOrTag, Encodable2718},
	hex::FromHex as _,
	network::{EthereumWallet, TransactionBuilder},
	primitives::{Address, Bytes, TxHash, U256, address, utils::parse_ether},
	providers::Provider,
	rpc::types::TransactionRequest,
	signers::local::PrivateKeySigner,
	sol,
	sol_types::SolCall,
};
use base64::prelude::*;
use color_eyre::eyre::{Context as _, eyre};
use intu_sdk::abi::IntuVault::proposeTransactionCall;
use tracing::{debug, error, info, trace, warn};

#[path = "tracing.rs"]
mod app_tracing;

const VAULT: Address = address!("0x14E559E06a524fb546e2eaceF79A11aEC85c16C2");
const SEPOLIA_ARBITRUM_RPC: &str = "https://sepolia-rollup.arbitrum.io/rpc";

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,intu_sdk=trace,intu_cli=trace")?;

	let signer: PrivateKeySigner =
		include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/private-key"))
			.trim()
			.parse()?;
	let wallet = EthereumWallet::new(signer);
	let provider = alloy::providers::ProviderBuilder::new()
		.wallet(wallet.clone())
		.connect(SEPOLIA_ARBITRUM_RPC)
		.await?;

	let intu_contract = intu_sdk::abi::IntuVault::new(VAULT, &provider);

	{
		let example_propose = provider
			.get_transaction_by_hash(TxHash::from_str(
				// "0xe6b5ee3c9a20879861aa20aaf2f441dc06bb1a0fa5bdc6c4882807001a25a569", // arb-sep-eth 0.005 to 0x45bFd673ad67682CCBe47C38234d7CE4aD288a5b
				"0x4be07fce98ab1ca937ba9fe158bb7dc46b92f635050cabaa614c9d5d01f1da5f", // sep-eth 0.005 to 0x45bFd673ad67682CCBe47C38234d7CE4aD288a5b
				                                                                      // "0xc9bfabe550c0304d1c137ad574fec8bd6d1216b0d6e32fdbcc8697e3112289fa",
			)?)
			.await?
			.unwrap();
		debug!(?example_propose);
		let input = example_propose.input();
		let args = proposeTransactionCall::abi_decode(&input)?;
		info!(?args);

		// let example_proposed_tx = "7ReEBfXhAIJSCJRFv9ZzrWdoLMvkfDgjTXzkrSiKW4cRw3k34IAAgIMGbu6AgA==";
		let example = args.transactionInfo;
		let example_proposed_tx = BASE64_STANDARD.decode(example)?;
		debug!(?example_proposed_tx);

		use alloy::consensus::transaction::RlpEcdsaDecodableTx;
		use alloy::eips::Decodable2718;
		// use alloy::rlp::decode::Decodable;
		let tx: EthereumTxEnvelope<alloy::consensus::TxEip4844Variant> =
			EthereumTxEnvelope::decode_2718(&mut &example_proposed_tx[..])?;
		debug!(final_tx = ?tx);

		let experiment: Vec<u8> = Vec::from_hex("0x45bfd673ad67682ccbe47c38234d7ce4ad288a5b")?;
		info!(?experiment);

		return Ok(());
	}

	let vault_info = intu_contract.vaultInfos().call().await?;
	info!("Found info for vault in Rust: {:?}", vault_info);

	let transactions = intu_contract.transactions(U256::from(84)).call().await?;
	info!(?transactions);

	let me = address!("0xEA428233445A5Cf500B9d5c91BcA6E7B887f7D70");
	let amount = parse_ether("0.5")?;
	let nonce = provider.get_transaction_count(VAULT).await?;
	let gas_fees = provider
		.get_fee_history(1, BlockNumberOrTag::Latest, &[])
		.await?;
	let gas_price: u128 = gas_fees
		.latest_block_base_fee()
		.ok_or(eyre!("Couldn't get latest block base fee"))?;
	let gas = 100_000;
	let tx: EthereumTxEnvelope<alloy::consensus::TxEip4844Variant> = TransactionRequest::default()
		.with_to(me)
		.with_value(amount)
		.with_chain_id(421614)
		.with_nonce(nonce)
		.with_gas_price(gas_price)
		.with_gas_limit(gas)
		.build(&wallet)
		.await?;
	// .build_unsigned()?;

	debug!(?tx);

	let tx = tx.encoded_2718();
	// let tx = tx.encoded_for_signing();
	debug!(?tx);
	let tx = BASE64_STANDARD.encode(tx);

	debug!(?tx);

	// let test_transaction = TransactionBuilder {};
	let pending = intu_contract
		.proposeTransaction(tx, String::from("first time?"))
		.send()
		.await
		.wrap_err("Failed to propose transaction")?;
	let done = pending
		.get_receipt()
		.await
		.wrap_err("Failed to get receipt for proposed transaction")?;
	info!(?done, "Proposed transaction!");

	Ok(())
}
