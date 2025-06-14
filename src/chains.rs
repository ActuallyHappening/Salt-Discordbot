use alloy::{hex::FromHexError, primitives::{utils::parse_ether, Address, U256}};
use twilight_interactions::command::{CommandModel, CreateCommand};
use url::Url;

use crate::{env::Env, prelude::*};

pub trait BlockchainListing {
	fn chain_id(&self) -> u64;
	fn chain_name(&self) -> &'static str;
	fn native_token_name(&self) -> &'static str;
}

pub(super) trait FaucetBlockchain: BlockchainListing {
	fn rpc_url(&self, env: &Env) -> Url;

	fn faucet_amount(&self) -> U256;

	fn address_str(&self) -> &str;
	fn address(&self) -> Result<Address, FromHexError> {
		self.address_str().parse()
	}
}

#[derive(Debug, Clone)]
pub(super) enum SupportedChain {
	SomniaShannon(SomniaShannon),
	SepoliaEtherium(SepoliaEthereum),
	SepoliaArbitrum(SepoliaArbitrum),
	PolygonAmoy(PolygonAmoy),
}

impl BlockchainListing for SupportedChain {
	fn chain_id(&self) -> u64 {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.chain_id(),
			SupportedChain::PolygonAmoy(chain) => chain.chain_id(),
			SupportedChain::SepoliaArbitrum(chain) => chain.chain_id(),
			SupportedChain::SepoliaEtherium(chain) => chain.chain_id(),
		}
	}

	fn chain_name(&self) -> &'static str {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.chain_name(),
			SupportedChain::PolygonAmoy(chain) => chain.chain_name(),
			SupportedChain::SepoliaArbitrum(chain) => chain.chain_name(),
			SupportedChain::SepoliaEtherium(chain) => chain.chain_name(),
		}
	}

	fn native_token_name(&self) -> &'static str {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.native_token_name(),
			SupportedChain::PolygonAmoy(chain) => chain.native_token_name(),
			SupportedChain::SepoliaArbitrum(chain) => chain.native_token_name(),
			SupportedChain::SepoliaEtherium(chain) => chain.native_token_name(),
		}
	}
}

impl FaucetBlockchain for SupportedChain {
	fn rpc_url(&self, env: &Env) -> Url {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.rpc_url(env),
			SupportedChain::PolygonAmoy(chain) => chain.rpc_url(env),
			SupportedChain::SepoliaArbitrum(chain) => chain.rpc_url(env),
			SupportedChain::SepoliaEtherium(chain) => chain.rpc_url(env),
		}
	}

	fn address_str(&self) -> &str {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.address_str(),
			SupportedChain::PolygonAmoy(chain) => chain.address_str(),
			SupportedChain::SepoliaArbitrum(chain) => chain.address_str(),
			SupportedChain::SepoliaEtherium(chain) => chain.address_str(),
		}
	}

	fn faucet_amount(&self) -> U256 {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.faucet_amount(),
			SupportedChain::PolygonAmoy(chain) => chain.faucet_amount(),
			SupportedChain::SepoliaArbitrum(chain) => chain.faucet_amount(),
			SupportedChain::SepoliaEtherium(chain) => chain.faucet_amount(),
		}
	}
}

/// Faucet 0.01 on Somnia Shannon STT tokens
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "somnia-shannon")]
pub struct SomniaShannon {
	/// Your personal wallet address
	pub address: String,
}

impl BlockchainListing for SomniaShannon {
	fn chain_id(&self) -> u64 {
		50312
	}

	fn chain_name(&self) -> &'static str {
		"Somnia Shannon"
	}

	fn native_token_name(&self) -> &'static str {
		"STT"
	}
}

impl FaucetBlockchain for SomniaShannon {
	fn rpc_url(&self, env: &Env) -> Url {
		env.somnia_shannon_rpc_endpoint.to_owned()
	}

	fn faucet_amount(&self) -> U256 {
		parse_ether("0.01").unwrap()
	}

	fn address_str(&self) -> &str {
		&self.address
	}
}

/// Faucet 0.005ETH on Ethereum Sepolia
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-eth")]
pub struct SepoliaEthereum {
	/// Your personal wallet address
	pub address: String,
}

impl BlockchainListing for SepoliaEthereum {
	fn chain_id(&self) -> u64 {
		11155111
	}
	fn chain_name(&self) -> &'static str {
		"Sepolia Ethereum"
	}
	fn native_token_name(&self) -> &'static str {
		"ETH"
	}
}

impl FaucetBlockchain for SepoliaEthereum {
	fn rpc_url(&self, env: &Env) -> Url {
		env.sepolia_ethereum_rpc_endpoint.to_owned()
	}

	fn faucet_amount(&self) -> U256 {
		parse_ether("0.005").unwrap()
	}

	fn address_str(&self) -> &str {
		&self.address
	}
}

/// Faucet 0.005ETH on Arbitrum Sepolia (gas for salt orchestration)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-arb-eth")]
pub struct SepoliaArbitrum {
	/// Your personal wallet address
	pub address: String,
}

impl BlockchainListing for SepoliaArbitrum {
	fn chain_id(&self) -> u64 {
		421614
	}
	fn chain_name(&self) -> &'static str {
		"Sepolia Arbitrum"
	}
	fn native_token_name(&self) -> &'static str {
		"ETH"
	}
}

impl FaucetBlockchain for SepoliaArbitrum {
	fn rpc_url(&self, env: &Env) -> Url {
		env.sepolia_arbitrum_rpc_endpoint.to_owned()
	}

	fn faucet_amount(&self) -> U256 {
		parse_ether("0.005").unwrap()
	}
	fn address_str(&self) -> &str {
		&self.address
	}
}

/// Faucet 0.005ETH on Polygon Amoy
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "polygon-amoy")]
pub struct PolygonAmoy {
	/// Your personal wallet address
	pub address: String,
}

impl BlockchainListing for PolygonAmoy {
	fn chain_id(&self) -> u64 {
		80002
	}
	fn chain_name(&self) -> &'static str {
		"Polygon Amoy"
	}
	fn native_token_name(&self) -> &'static str {
		"AMOY"
	}
}

impl FaucetBlockchain for PolygonAmoy {
	fn rpc_url(&self, env: &Env) -> Url {
		env.polygon_amoy_rpc_endpoint.to_owned()
	}

	fn faucet_amount(&self) -> U256 {
		parse_ether("0.005").unwrap()
	}

	fn address_str(&self) -> &str {
		&self.address
	}
}
