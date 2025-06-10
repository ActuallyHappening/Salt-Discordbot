use alloy::sol;

sol!(
	#[sol(rpc, abi)]
	#[derive(Debug)]
	IntuVault,
	concat!(env!("CARGO_MANIFEST_DIR"), "/abi/Vault.json")
);
