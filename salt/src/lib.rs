pub mod prelude;

use std::{net::SocketAddrV4, process::ExitStatus};

use alloy_primitives::{
	Address, U256,
	utils::{ParseUnits, Unit, UnitsError, parse_ether},
};
use camino::FromPathBufError;
use cli::{AsyncCommand, Output};
use color_eyre::{
	Section,
	eyre::{Context as _, eyre},
};
use hex::DisplayHex as _;
use tokio::{io::AsyncReadExt as _, sync::oneshot};
use url::Url;
use which::which;

use crate::prelude::*;

pub use salt::*;

pub use errors::*;
mod cli;
mod errors;
mod salt;
mod which;
