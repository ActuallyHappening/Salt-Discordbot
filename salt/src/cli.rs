#![allow(dead_code)]

use std::process::{ExitStatus, Stdio};

use crate::prelude::*;

pub(crate) struct Command(std::process::Command);

pub(crate) struct AsyncCommand(tokio::process::Command);

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

impl AsyncCommand {
	pub fn pure(cmd: Utf8PathBuf) -> Result<AsyncCommand> {
		if !cmd.exists() {
			return Err(Error::ExecutableFileDoesntExist(cmd));
		}
		let mut cmd = tokio::process::Command::new(cmd);
		cmd.env_clear();
		Ok(AsyncCommand(cmd))
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

	pub async fn run_and_wait(mut self) -> Result<()> {
		self.pre_logging();

		let status = self.0.status().await.map_err(Error::FailedToExecute)?;

		if !status.success() {
			return Err(Error::SubprocessExitedBadly(status));
		}
		Ok(())
	}

	pub async fn run_and_wait_for_output(mut self) -> Result<Output> {
		self.pre_logging();

		let output: Output = self
			.0
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.spawn()
			.map_err(Error::FailedToExecute)?
			.wait_with_output()
			.await
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
