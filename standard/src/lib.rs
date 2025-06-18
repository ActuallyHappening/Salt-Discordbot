use alloy::primitives::{Address, address};

pub mod prelude {
	pub use tracing::{debug, error, info, trace, warn};
}

const CONTRACT_ADDRESS: Address = address!("0x44e7525cf9d56733d08fc98bcd750d504fce91ec");

// sell my SST to USDC
// https://shannon-explorer.somnia.network/tx/0x367e088caf50e0d2ef3f7fe8fd0ce45ddc044107d1876c6b18dd31baef8b5a5e

/// USDC token address
/// https://somnia-testnet-ponder-v5.standardweb3.com/api/token/0x0ED782B8079529f7385c3eDA9fAf1EaA0DbC6a17
pub const USDC: Address = address!("0x0ED782B8079529f7385c3eDA9fAf1EaA0DbC6a17");

/// https://somnia-testnet-ponder-v5.standardweb3.com/api/token/0xb35a7935F8fbc52fB525F16Af09329b3794E8C42
pub const WSOL: Address = address!("0xb35a7935F8fbc52fB525F16Af09329b3794E8C42");

/// https://somnia-testnet-ponder-v5.standardweb3.com/api/token/0x54597df4E4A6385B77F39d458Eb75443A8f9Aa9e
pub const WBTC: Address = address!("0x54597df4E4A6385B77F39d458Eb75443A8f9Aa9e");

// wss://somnia-testnet-websocket-v5.standardweb3.com/
// https://learn.standardweb3.com/apps/spot/for-developers/websocket-streams

pub mod abis {
	pub mod matching_engine {
		use alloy::sol;

		sol! {
			#[sol(rpc, abi)]
			#[derive(Debug)]
			MatchingEngine,
			concat!(env!("CARGO_MANIFEST_DIR"), "/abi/MatchingEngine.json"),
		}
	}
}
