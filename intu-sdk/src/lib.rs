/// Generated contract ABI code.
/// Wow Rust is truly awesome!
pub mod abi {
	use alloy::sol;
	sol!(
		#[sol(rpc, abi)]
		#[derive(Debug)]
		IntuVault,
		concat!(env!("CARGO_MANIFEST_DIR"), "/abi/Vault.json")
	);
}

/// Here for completeness not usability
pub mod other_abi {
	pub mod factory {
		use alloy::sol;
		sol!(
			#[sol(rpc, abi)]
			#[derive(Debug)]
			VaultFactory,
			concat!(env!("CARGO_MANIFEST_DIR"), "/abi/VaultFactory.json")
		);
	}

	pub mod fee {
		use alloy::sol;
		sol!(
			#[sol(rpc, abi)]
			#[derive(Debug)]
			VaultFactory,
			concat!(env!("CARGO_MANIFEST_DIR"), "/abi/Fee.json")
		);
	}
}

pub mod rpc;
