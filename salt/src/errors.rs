use std::process::ExitStatus;

use camino::FromPathBufError;

use crate::{cli::Output, prelude::*};

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error(
		"Couldn't find appropriate default directory: https://docs.rs/dirs/latest/dirs/fn.data_dir.html or https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html"
	)]
	NoStandardDirectoryFound,

	#[error("{0}")]
	Camino(#[from] FromPathBufError),

	#[error("Executable file doesn't exist")]
	ExecutableFileDoesntExist(Utf8PathBuf),

	#[error("Failed to execute subprocess: {0}")]
	FailedToExecute(#[source] std::io::Error),

	#[error("Something went wrong collecting salt-asset-manager logs: {0}")]
	LiveLogging(#[source] color_eyre::Report),

	#[error("Subprocess exited badly: {0:?}")]
	SubprocessExitedBadly(ExitStatus),

	#[error("Subprocess exited badly with exit status {0}")]
	SubprocessExitedBadlyWithOutput(Output),

	#[error("Couldn't make anonymous pipe: {0}")]
	CouldntMakeAnonymousePipe(#[source] std::io::Error),

	#[error(
		"Expected `{bin_name}` binary to be in PATH environment variable or finable with which https://docs.rs/which/latest/which/fn.which.html ({err_msg}): {which}"
	)]
	Which {
		bin_name: String,
		err_msg: String,
		#[source]
		which: ::which::Error,
	},

	#[error("Can't serialize as JSON: {0}")]
	SerdeJson(#[source] color_eyre::Report),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
