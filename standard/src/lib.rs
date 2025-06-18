use alloy::primitives::{Address, address};

pub mod prelude {
	pub use ::nonzero_lit::*;
	pub use alloy::primitives::{Address, address};
	pub(crate) use color_eyre::{eyre::WrapErr as _, eyre::eyre};
	pub use core::num::NonZero;
	pub(crate) use tracing::{debug, error, info, trace, warn};
	pub(crate) use url::Url;
}

#[path = "tracing.rs"]
pub(crate) mod app_tracing;

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

pub mod apis {
	pub mod rest;
}

pub mod abis;
