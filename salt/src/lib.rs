pub mod prelude {
	#![allow(unused_imports)]
	pub use crate::Salt;

	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use crate::{Error, Result};
	pub(crate) use camino::{Utf8Path, Utf8PathBuf};
}

use std::process::ExitStatus;

use camino::FromPathBufError;
use cli::Command;
use git::Git;
use url::Url;
use which::which;

use crate::prelude::*;

pub struct Salt {
	project_folder: Utf8PathBuf,
	config: SaltConfig,
}

#[derive(Clone, serde::Deserialize)]
pub struct SaltConfig {
	#[serde(rename = "PRIVATE_KEY")]
	pub private_key: String,

	#[serde(rename = "ORCHESTRATION_NETWORK_RPC_NODE_URL")]
	pub orchestration_network_rpc_node: Url,

	#[serde(rename = "BROADCASTING_NETWORK_RPC_NODE_URL")]
	pub broadcasting_network_rpc_node: Url,

	#[serde(rename = "BROADCASTING_NETWORK_ID")]
	pub broadcasting_network_id: String,
}

impl std::fmt::Debug for SaltConfig {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let map = self.clone().iter();
		let mut fmt = f.debug_struct("SaltConfig");
		let blacklisted_fields = ["PRIVATE_KEY"];
		for (key, value) in map {
			if blacklisted_fields.contains(&key) {
				fmt.field(key, &"redacted".to_owned());
			} else {
				fmt.field(key, &value);
			}
		}
		fmt.finish()
	}
}

impl SaltConfig {
	fn iter(self) -> impl IntoIterator<Item = (&'static str, String)> {
		[
			("PRIVATE_KEY", self.private_key),
			(
				"ORCHESTRATION_NETWORK_RPC_NODE_URL",
				self.orchestration_network_rpc_node.to_string(),
			),
			(
				"BROADCASTING_NETWORK_RPC_NODE_URL",
				self.broadcasting_network_rpc_node.to_string(),
			),
			("BROADCASTING_NETWORK_ID", self.broadcasting_network_id),
		]
	}
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

	#[error(
		"Expected `{bin_name}` binary to be in PATH environment variable or finable with which https://docs.rs/which/latest/which/fn.which.html ({err_msg}): {which}"
	)]
	Which {
		bin_name: String,
		err_msg: String,
		which: ::which::Error,
	},
}

pub type Result<T> = core::result::Result<T, Error>;

impl Salt {
	fn default_project_path() -> Result<Utf8PathBuf> {
		let dir = dirs::data_local_dir()
			.or(dirs::data_dir())
			.ok_or(Error::NoStandardDirectoryFound)?;
		let path = Utf8PathBuf::try_from(dir)?;
		Ok(path.join("salt-asset-manager"))
	}

	pub fn new(config: SaltConfig) -> Result<Salt> {
		let salt = Salt {
			project_folder: Salt::default_project_path()?,
			config,
		};

		salt.init()?;

		Ok(salt)
	}

	pub fn transaction(
		&self,
		amount: String,
		vault_address: String,
		recipient_address: String,
	) -> Result<()> {
		self.cmd([
			"-amount".into(),
			amount,
			"-vault-address".into(),
			vault_address,
			"-recipient-address".into(),
			recipient_address,
		])?
		.run_and_wait()?;
		Ok(())
	}

	pub fn broadcasting_network_id(&self) -> String {
		self.config.broadcasting_network_id.clone()
	}

	/// git pull && deno install && nu fix.nu
	fn init(&self) -> Result<()> {
		let git = self.git()?;
		git.ensure_latest_branch(
			Url::parse("https://github.com/ActuallyHappening/salt-asset-manager").unwrap(),
			"master",
		)?;

		let deno = Salt::deno()?;
		cli::Command::pure(deno)?
			.with_cwd(self.project_folder.clone())
			.with_args(["install".into()])
			.run_and_wait()?;

		if self.project_folder.join("fix.nu").exists() {
			debug!("Detected fix.nu, running this after deno install");
			// run fix.nu
			let nu = which(
				"nu",
				"required shell to run fix.nu, see https://www.nushell.sh/book/installation.html#package-managers",
			)?;
			cli::Command::pure(nu)?
				.with_cwd(self.project_folder.clone())
				.with_args(["fix.nu".into()])
				.run_and_wait()?;
		}

		info!(
			"Successfully initialized/updated git checkout at {} ready for runtime consumption",
			self.project_folder
		);

		Ok(())
	}

	fn deno() -> Result<Utf8PathBuf> {
		which("deno", "required javascript runtime")
	}

	fn cmd(&self, args: impl IntoIterator<Item = String>) -> Result<Command> {
		let cmd = cli::Command::pure(Salt::deno()?)?
			.with_cwd(self.project_folder.clone())
			.with_args(
				[
					"run",
					"--unstable-sloppy-imports",
					"-A",
					"src/index.ts",
					"--",
					"-use-cli-only",
				]
				.into_iter()
				.map(String::from),
			)
			.with_args(args)
			.with_envs(self.config.clone().iter());
		Ok(cmd)
	}

	fn git(&self) -> Result<Git> {
		Ok(Git::new(self.project_folder.to_owned())?)
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

		pub fn with_envs(mut self, envs: impl IntoIterator<Item = (&'static str, String)>) -> Self {
			self.0.envs(envs);
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
		project_folder: Utf8PathBuf,
	}

	impl Git {
		pub fn new(path: Utf8PathBuf) -> Result<Self> {
			Ok(Self {
				project_folder: path,
			})
		}

		fn cmd(&self) -> Result<Command> {
			let git = which("git", "required runtime dependency")?;
			let cmd = Command::pure(git)?.with_cwd(self.project_folder.to_owned());
			Ok(cmd)
		}

		pub fn ensure_latest_branch(&self, repository_url: Url, branch: &str) -> Result<()> {
			if self.project_folder.exists() {
				// assume already checked out
				self.pull()?;
			} else {
				self.clone(repository_url)?;
			}
			self.checkout(branch)?;
			Ok(())
		}

		fn pull(&self) -> Result<()> {
			debug!("Running `git pull` in directory {}", &self.project_folder);
			self.cmd()?.with_args(["pull".into()]).run_and_wait()
		}

		fn clone(&self, repository_url: Url) -> Result<()> {
			let mut parent_folder = self.project_folder.clone();
			if !parent_folder.pop() {
				panic!("self.project_folder has no parent? Why is the data dir at / ?");
			}
			debug!(
				"Running `git clone {}` in directory {}",
				repository_url, &parent_folder
			);
			self.cmd()?
				.with_cwd(parent_folder)
				.with_args([
					"clone".into(),
					repository_url.to_string(),
					self.project_folder.to_string(),
				])
				.run_and_wait()
		}

		fn checkout(&self, branch: &str) -> Result<()> {
			self.cmd()?
				.with_args(["checkout".into(), branch.into()])
				.run_and_wait()
		}
	}
}

mod which {
	use crate::prelude::*;

	pub fn which(name: &'static str, err_msg: impl Into<String>) -> Result<Utf8PathBuf> {
		let path = ::which::which(name).map_err(|which| Error::Which {
			bin_name: name.to_owned(),
			err_msg: err_msg.into(),
			which,
		})?;
		Ok(Utf8PathBuf::try_from(path)?)
	}
}
