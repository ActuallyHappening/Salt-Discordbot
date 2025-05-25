pub mod prelude {
	#![allow(unused_imports)]
	pub use crate::Salt;

	pub(crate) use camino::{Utf8Path, Utf8PathBuf};
}

use camino::FromPathBufError;

use crate::prelude::*;

pub struct Salt {
	path: Utf8PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(
		"Couldn't find appropriate default director: https://docs.rs/dirs/latest/dirs/fn.data_dir.html or https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html"
	)]
	NoStandardDirectoryFound,

	#[error("{0}")]
	Camino(#[from] FromPathBufError),
}

pub type Result<T> = core::result::Result<T, Error>;

impl Salt {
	fn default_path() -> Result<Utf8PathBuf> {
		let dir = dirs::data_local_dir()
			.or(dirs::data_dir())
			.ok_or(Error::NoStandardDirectoryFound);
		Ok(Utf8PathBuf::try_from(dir?)?)
	}
	
	pub fn new() -> Result<Salt> {
		Ok(Salt {
			path: Salt::default_path()?,
		})
	}
}
