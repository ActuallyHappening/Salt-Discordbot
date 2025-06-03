use twilight_interactions::command::{CommandModel, CreateCommand};
use url::Url;

use crate::{env::Env, prelude::*};

pub(super) trait FaucetBlockchain {
	fn info(&self, env: &Env) -> BlockchainInfo;

	fn faucet_amount(&self) -> String {
		String::from("0.005")
	}

	fn address(&self) -> String;
}

pub(super) struct BlockchainInfo {
	pub chain_id: u64,
	pub rpc_url: Url,
	pub token_name: &'static str,
	pub chain_name: &'static str,
}

#[derive(Debug, Clone)]
pub(super) enum SupportedChain {
	SomniaShannon(SomniaShannon),
	SepoliaEtherium(SepoliaEtherium),
	SepoliaArbitrum(SepoliaArbitrum),
	PolygonAmoy(PolygonAmoy),
}

impl FaucetBlockchain for SupportedChain {
	fn info(&self, env: &Env) -> BlockchainInfo {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.info(env),
			SupportedChain::PolygonAmoy(chain) => chain.info(env),
			SupportedChain::SepoliaArbitrum(chain) => chain.info(env),
			SupportedChain::SepoliaEtherium(chain) => chain.info(env),
		}
	}

	fn address(&self) -> String {
		match self {
			SupportedChain::SomniaShannon(chain) => chain.address(),
			SupportedChain::PolygonAmoy(chain) => chain.address(),
			SupportedChain::SepoliaArbitrum(chain) => chain.address(),
			SupportedChain::SepoliaEtherium(chain) => chain.address(),
		}
	}

	fn faucet_amount(&self) -> String {
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

impl FaucetBlockchain for SomniaShannon {
	fn info(&self, env: &Env) -> BlockchainInfo {
		BlockchainInfo {
			chain_id: 50312,
			rpc_url: env.somnia_shannon_rpc_endpoint.clone(),
			token_name: "STT",
			chain_name: "Somnia Shannon",
		}
	}

	fn faucet_amount(&self) -> String {
		String::from("0.01")
	}

	fn address(&self) -> String {
		self.address.clone()
	}
}

/// Faucet 0.005ETH on Ethereum Sepolia
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-eth")]
pub struct SepoliaEtherium {
	/// Your personal wallet address
	pub address: String,
}

impl FaucetBlockchain for SepoliaEtherium {
	fn info(&self, env: &Env) -> BlockchainInfo {
		BlockchainInfo {
			chain_id: 11155111,
			rpc_url: env.sepolia_etherium_rpc_endpoint.clone(),
			token_name: "ETH",
			chain_name: "Sepolia Ethereum",
		}
	}

	fn address(&self) -> String {
		self.address.clone()
	}
}

/// Faucet 0.005ETH on Arbitrum Sepolia (gas for salt orchestration)
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "sepolia-arb-eth")]
pub struct SepoliaArbitrum {
	/// Your personal wallet address
	pub address: String,
}

impl FaucetBlockchain for SepoliaArbitrum {
	fn info(&self, env: &Env) -> BlockchainInfo {
		BlockchainInfo {
			chain_id: 421614,
			rpc_url: env.sepolia_arbitrum_rpc_endpoint.clone(),
			token_name: "ETH",
			chain_name: "Sepolia Arbitrum",
		}
	}

	fn address(&self) -> String {
		self.address.clone()
	}
}

/// Faucet 0.005ETH on Polygon Amoy
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "polygon-amoy")]
pub struct PolygonAmoy {
	/// Your personal wallet address
	pub address: String,
}

impl FaucetBlockchain for PolygonAmoy {
	fn info(&self, env: &Env) -> BlockchainInfo {
		BlockchainInfo {
			chain_id: 80002,
			rpc_url: env.polygon_amoy_rpc_endpoint.clone(),
			token_name: "AMOY",
			chain_name: "Polygon Amoy",
		}
	}

	fn address(&self) -> String {
		self.address.clone()
	}
}
