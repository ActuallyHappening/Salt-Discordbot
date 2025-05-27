pub mod prelude {
	#![allow(unused_imports)]
	pub use crate::Salt;

	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use crate::{Error, Result};
	pub(crate) use camino::{Utf8Path, Utf8PathBuf};
}

use std::process::ExitStatus;

use camino::FromPathBufError;
use cli::{Command, Output};
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
	pub broadcasting_network_id: u64,
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
			(
				"BROADCASTING_NETWORK_ID",
				self.broadcasting_network_id.to_string(),
			),
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
	SubprocessExitedBadly(ExitStatus),

	#[error("Subprocess exited badly with exit status {0}")]
	SubprocessExitedBadlyWithOutput(Output),

	#[error("Couldn't make anonymous pipe: {0}")]
	CouldntMakeAnonymousePipe(std::io::Error),

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

	#[tracing::instrument(name = "salt_sdk::transaction", skip_all)]
	pub fn transaction(
		&self,
		amount: &str,
		vault_address: &str,
		recipient_address: &str,
	) -> Result<Output> {
		debug!("Beginning transaction ...");
		let output = self
			.cmd([
				"-amount",
				amount,
				"-vault-address",
				vault_address,
				"-recipient-address",
				recipient_address,
			])?
			.run_and_wait_for_output()?;
		debug!("Finished transaction.");

		Ok(output)
	}

	pub fn broadcasting_network_id(&self) -> u64 {
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
			.with_args(["install"])
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
				.with_args(["fix.nu"])
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

	fn cmd(&self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<Command> {
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
	use std::process::{ExitStatus, Stdio};

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

		pub fn with_args(mut self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
			// allocates them all as strings,
			// but what can you do? this is a type-level restriction,
			// it is true that
			// impl AsRef<str>: impl AsRef<OsStr>
			// for all T
			self.0.args(args.into_iter().map(|s| s.as_ref().to_owned()));
			self
		}

		pub fn with_envs(mut self, envs: impl IntoIterator<Item = (&'static str, String)>) -> Self {
			self.0.envs(envs);
			self
		}

		/// Hides PRIVATE_KEY
		fn debug(&self) -> String {
			let mut initial = format!("{:?}", self.0);
			let re = regex::Regex::new(r#"PRIVATE_KEY="([a-zA-z0-9]+)""#).unwrap();
			if let Some(find) = re.find(&initial) {
				initial = initial.replace(find.as_str(), r#"PRIVATE_KEY="redacted""#);
			}
			initial
		}

		fn pre_logging(&self) {
			trace!("Running command {}", self.debug());
		}

		pub fn run_and_wait(mut self) -> Result<()> {
			self.pre_logging();

			let status = self.0.status().map_err(Error::FailedToExecute)?;

			if !status.success() {
				return Err(Error::SubprocessExitedBadly(status));
			}
			Ok(())
		}

		/// Pipes to terminal and collects
		pub fn run_and_wait_for_output(mut self) -> Result<Output> {
			self.pre_logging();

			let output: Output = self
				.0
				.stdout(Stdio::piped())
				.stderr(Stdio::piped())
				.spawn()
				.map_err(Error::FailedToExecute)?
				.wait_with_output()
				.map_err(Error::FailedToExecute)?
				.into();

			if !output.status.success() {
				return Err(Error::SubprocessExitedBadlyWithOutput(output));
			}

			Ok(output)
		}
	}

	#[derive(Debug)]
	pub struct Output {
		pub status: ExitStatus,
		pub stdout: String,
		pub stderr: String,
	}

	/// Display impl is status \n stderr \n stdout
	impl std::fmt::Display for Output {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(
				f,
				"{:?}:\nStderr:\n{}\nStdout:\n{}",
				self.status, self.stderr, self.stdout
			)
		}
	}

	impl From<std::process::Output> for Output {
		fn from(value: std::process::Output) -> Self {
			let stdout = String::from_utf8_lossy(&value.stdout);
			let stderr = String::from_utf8_lossy(&value.stderr);
			Self {
				status: value.status,
				stdout: stdout.into(),
				stderr: stderr.into(),
			}
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
			self.cmd()?.with_args(["pull"]).run_and_wait()
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
			self.cmd()?.with_args(["checkout", branch]).run_and_wait()
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
