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
		if !self.project_folder.exists() {
			self.clone(repository_url)?;
		}
		self.checkout(branch)?;
		self.pull()?;
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
