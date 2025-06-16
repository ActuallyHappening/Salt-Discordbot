use alloy::primitives::{Address, address};

pub mod prelude {
	pub use tracing::{debug, error, info, trace, warn};
}

const CONTRACT_ADDRESS: Address = address!("0x44e7525cf9d56733d08fc98bcd750d504fce91ec");

// sell my SST to USDC
// https://shannon-explorer.somnia.network/tx/0x367e088caf50e0d2ef3f7fe8fd0ce45ddc044107d1876c6b18dd31baef8b5a5e

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
