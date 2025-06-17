use alloy::providers::ProviderBuilder;
use tracing::{debug, error, info, trace, warn};

#[path = "tracing.rs"]
mod app_tracing;

// Generate the contract bindings for the ERC20 interface.
alloy::sol! {
	// The `rpc` attribute enables contract interaction via the provider.
	#[sol(rpc, abi)]
	contract ERC20 {
		function name() public view returns (string);
		function symbol() public view returns (string);
		function decimals() public view returns (uint8);
		function totalSupply() public view returns (uint256);
		function balanceOf(address account) public view returns (uint256);
		function transfer(address recipient, uint256 amount) public returns (bool);
		function allowance(address owner, address spender) public view returns (uint256);
		function approve(address spender, uint256 amount) public returns (bool);
		function transferFrom(address sender, address recipient, uint256 amount) public returns (bool);

		event Transfer(address indexed from, address indexed to, uint256 value);
		event Approval(address indexed owner, address indexed spender, uint256 value);
	}
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
	app_tracing::install_tracing("info,standard=trace")?;
	trace!("Started tracing");

	let provider = ProviderBuilder::new()
		.connect("https://dream-rpc.somnia.network/")
		.await?;
	let usdc = ERC20::new(standard::USDC, provider);

	let name = usdc.name().call().await?;
	let total_supply = usdc.totalSupply().call().await?;

	info!(?name, ?total_supply);

	Ok(())
}
