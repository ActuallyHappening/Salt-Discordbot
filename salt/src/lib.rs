pub mod prelude {
	#![allow(unused_imports)]
	pub use crate::Salt;

	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use crate::{Error, Result};
	pub(crate) use camino::{Utf8Path, Utf8PathBuf};
}

use std::process::ExitStatus;

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

	#[error("Executable file doesn't exist")]
	ExecutableFileDoesntExist(Utf8PathBuf),

	#[error("{0}")]
	FailedToExecute(#[from] bossy::Error),

	#[error("Subprocess exited badly: {0:?}")]
	SubprocessExited(ExitStatus),

	#[error("{0}")]
	Which(#[from] ::which::Error),
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

mod cli {
	use crate::prelude::*;

	#[derive(Clone)]
	pub struct Command(bossy::Command);

	impl Command {
		pub fn pure(cmd: Utf8PathBuf) -> Result<Command> {
			if !cmd.exists() {
				return Err(Error::ExecutableFileDoesntExist(cmd));
			}
			Ok(Command(bossy::Command::pure(cmd)))
		}

		pub fn with_args(self, args: impl IntoIterator<Item = String>) -> Self {
			Self(self.0.with_args(args))
		}

		pub fn run_and_wait(mut self) -> Result<()> {
			let status = self.0.run_and_wait()?;
			if !status.success() {
				return Err(Error::SubprocessExited(status));
			}
			Ok(())
		}
	}
}

mod git {
	use url::Url;

	use crate::{cli::Command, prelude::*, which::which};

	#[derive(Clone)]
	pub struct Git {
		path: Utf8PathBuf,
		/// default cmd with no args always NB
		cmd: Command,
	}

	impl Git {
		pub fn new(path: Utf8PathBuf) -> Result<Self> {
			let git = which("git").inspect_err(|_| {
				error!(
					"A `git` binary available on PATH is a required runtime dependency of https://lib.rs/salt-sdk"
				)
			})?;
			let cmd = Command::pure(git)?;
			Ok(Git { cmd, path })
		}

		pub fn clone_and_or_pull(&self, repository_url: Url) {
			if self.path.exists() {
				// assume already checked out
				self.pull();
			}
		}

		fn pull(&self) -> Result<()> {
			self.cmd.clone().with_args(["pull".into()]).run_and_wait()
		}

		fn clone(&self, repository_url: Url) -> Result<()> {
			self.cmd
				.clone()
				.with_args([
					"clone".into(),
					repository_url.to_string(),
					self.path.to_string(),
				])
				.run_and_wait()
		}
	}
}

mod which {
	use crate::prelude::*;

	pub fn which(name: &'static str) -> Result<Utf8PathBuf> {
		let path = ::which::which(name).inspect_err(|_| {})?;
		Ok(Utf8PathBuf::try_from(path)?)
	}
}
