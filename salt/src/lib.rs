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
	FailedToExecute(std::io::Error),

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

	pub struct Command(std::process::Command);

	impl Command {
		pub fn pure(cmd: Utf8PathBuf) -> Result<Command> {
			if !cmd.exists() {
				return Err(Error::ExecutableFileDoesntExist(cmd));
			}
			let mut cmd = std::process::Command::new(cmd);
			cmd.env_clear();
			Ok(Command(cmd))
		}

		pub fn current_dir(&mut self, cwd: Utf8PathBuf) -> &mut Self {
			self.0.current_dir(cwd);
			self
		}

		pub fn with_cwd(mut self, cwd: Utf8PathBuf) -> Self {
			self.current_dir(cwd);
			self
		}

		pub fn with_args(mut self, args: impl IntoIterator<Item = String>) -> Self {
			self.0.args(args);
			self
		}

		pub fn run_and_wait(mut self) -> Result<()> {
			info!("Running command {:?}", self.0);
			let status = self.0.status().map_err(Error::FailedToExecute)?;
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

	pub struct Git {
		path: Utf8PathBuf,
	}

	impl Git {
		pub fn new(path: Utf8PathBuf) -> Result<Self> {
			Ok(Self { path })
		}

		fn cmd(&self) -> Result<Command> {
			let git = which("git").inspect_err(|_| {
				error!(
					"A `git` binary available on PATH is a required runtime dependency of https://lib.rs/salt-sdk"
				)
			})?;
			let cmd = Command::pure(git)?.with_cwd(self.path.to_owned());
			Ok(cmd)
		}

		pub fn clone_and_or_pull(&self, repository_url: Url) -> Result<()> {
			if self.path.exists() {
				// assume already checked out
				self.pull()
			} else {
				self.clone(repository_url)
			}
		}

		fn pull(&self) -> Result<()> {
			self.cmd()?.with_args(["pull".into()]).run_and_wait()
		}

		fn clone(&self, repository_url: Url) -> Result<()> {
			self.cmd()?
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
