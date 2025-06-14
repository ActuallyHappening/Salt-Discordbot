use std::{borrow::Borrow, marker::PhantomData};

use alloy::{
	hex::FromHexError,
	primitives::{Address, B256, U256, utils::parse_ether},
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use url::Url;

use crate::{
	chains::explorer::{BlockchainExplorer, ExplorableBlockchain, GenericBlockExplorer},
	env::Env,
	prelude::*,
};

mod explorer;

pub trait BlockchainListing {
	fn chain_id(&self) -> u64;
	fn chain_name(&self) -> &'static str;
	fn native_token_name(&self) -> &'static str;
}

pub(super) trait NativeFaucet: BlockchainListing + ExplorableBlockchain {
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

impl NativeFaucet for SupportedChain {
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

impl ExplorableBlockchain for SomniaShannon {
	type Explorer = GenericBlockExplorer<Self>;
	fn block_explorer(&self) -> Self::Explorer {
		GenericBlockExplorer::new("https://shannon-explorer.somnia.network/".parse().unwrap())
	}
}

impl BlockchainExplorer<SomniaShannon> for GenericBlockExplorer<SomniaShannon> {
	fn base(&self) -> Url {
		self.base.clone()
	}
}

impl NativeFaucet for SomniaShannon {
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

impl ExplorableBlockchain for SepoliaEthereum {
	type Explorer = GenericBlockExplorer<Self>;
	fn block_explorer(&self) -> Self::Explorer {
		GenericBlockExplorer::new("https://sepolia.etherscan.io".parse().unwrap())
	}
}

impl BlockchainExplorer<SepoliaEthereum> for GenericBlockExplorer<SepoliaEthereum> {
	fn base(&self) -> Url {
		self.base.clone()
	}
}

impl NativeFaucet for SepoliaEthereum {
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

impl NativeFaucet for SepoliaArbitrum {
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

impl NativeFaucet for PolygonAmoy {
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
