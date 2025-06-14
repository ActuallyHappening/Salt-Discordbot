use std::marker::PhantomData;

use alloy::primitives::B256;
use url::Url;

use crate::{chains::BlockchainListing, prelude::*};

pub trait ExplorableBlockchain: BlockchainListing + Sized {
	type Explorer: BlockchainExplorer<Self>;
	fn block_explorer(&self) -> Self::Explorer;
}

/// Assumes base + /tx + /:txhash to construct block explorer URL
pub struct GenericBlockExplorer<Chain> {
	_phantom: PhantomData<Chain>,
	pub base: Url,
}

pub trait BlockchainExplorer<Chain>
where
	Chain: BlockchainListing,
{
	fn base(&self) -> Url;

	fn transaction_explorer_url(&self, hex_hash: B256) -> color_eyre::Result<Url> {
		let mut url = self.base();
		url.path_segments_mut()
			.map_err(|_: ()| eyre!("Couldn't mutate url's path"))?
			.push("tx")
			.push(&hex_hash.to_string());
		Ok(url)
	}
}

impl<Chain> GenericBlockExplorer<Chain> {
	pub fn new(base: Url) -> GenericBlockExplorer<Chain> {
		GenericBlockExplorer {
			_phantom: PhantomData,
			base,
		}
	}

	pub fn adapt<NewChain>(self) -> GenericBlockExplorer<NewChain>
	where
		Chain: BlockchainListing,
		NewChain: BlockchainListing,
	{
		GenericBlockExplorer {
			_phantom: PhantomData,
			base: self.base,
		}
	}
}
