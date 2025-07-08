use std::{net::SocketAddrV4, time::Duration};

use crate::{
	cli::{self, Output},
	prelude::*,
};

use alloy::providers::{PendingTransactionConfig, Provider};
use alloy_primitives::{
	Address, TxHash, U256,
	utils::{ParseUnits, Unit},
};
pub use live_logging::*;
use tokio::sync::oneshot;
use ystd::time::FutureTimeoutExt as _;
mod live_logging;

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
	pub fn broadcasting_network_id(&self) -> u64 {
		self.config.broadcasting_network_id
	}

	/// git pull && deno install && nu fix.nu
	#[tracing::instrument(name = "init", skip_all)]
	fn init(&self) -> Result<()> {
		let git = self.git()?;
		git.ensure_latest_branch(
			Url::parse("https://github.com/ActuallyHappening/salt-asset-manager").unwrap(),
			if cfg!(debug_assertions) {
				"dev"
			} else {
				"master"
				// "8db97cbfc5367558cb960f8da783fc5c628db505" // FIXME
			},
		)?;

		let deno = Salt::deno()?;
		cli::Command::pure(deno)?
			.with_cwd(self.project_folder.clone())
			.with_args(["install"])
			.run_and_wait()?;

		if self.project_folder.join("patch.nu").exists() {
			debug!("Detected fix.nu, running this after deno install");
			// run fix.nu
			let nu = which(
				"nu",
				"required shell to run fix.nu, see https://www.nushell.sh/book/installation.html#package-managers",
			)?;
			cli::Command::pure(nu)?
				.with_cwd(self.project_folder.clone())
				.with_args(["patch.nu"])
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

	fn cmd(&self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Result<cli::AsyncCommand> {
		let cmd = cli::AsyncCommand::pure(Salt::deno()?)?
			.with_cwd(self.project_folder.clone())
			.with_args(
				["task", "--quiet", "start", "--", "-use-cli-only"]
					.into_iter()
					.map(String::from),
			)
			.with_args(args)
			.with_envs(self.config.clone().iter());
		Ok(cmd)
	}

	fn git(&self) -> Result<cli::Git> {
		cli::Git::new(self.project_folder.to_owned())
	}
}

pub struct TransactionInfo {
	pub amount: U256,
	pub vault_address: Address,
	pub recipient_address: Address,
	pub data: Vec<u8>,
	pub gas: GasEstimator,
	pub logging: LiveLogging,
	pub confirm_publish: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, Default)]
pub enum GasEstimator {
	#[default]
	Default,
	Mul(f64),
}

pub struct TransactionDone {
	pub hash: TxHash,
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	#[allow(unreachable_code, unused)]
	fn api_is_send() {
		fn is_send<T: Send>(_t: T) {}
		let salt: Salt = unimplemented!();
		is_send(logging(
			unimplemented!(),
			unimplemented!(),
			unimplemented!(),
			unimplemented!(),
		));
		is_send(salt.transaction(TransactionInfo {
			amount: todo!(),
			vault_address: todo!(),
			recipient_address: todo!(),
			data: todo!(),
			gas: todo!(),
			logging: LiveLogging::from_cb(|str| ()),
			confirm_publish: todo!(),
		}));
	}
}

impl Salt {
	///
	#[tracing::instrument(skip_all)]
	pub async fn transaction(&self, info: TransactionInfo) -> Result<TransactionDone> {
		debug!("Beginning transaction ...");

		let TransactionInfo {
			amount,
			vault_address,
			recipient_address,
			data,
			gas,
			logging: cb,
			confirm_publish,
		} = info;

		let addr: SocketAddrV4 = "127.0.0.1:0000".parse().unwrap();
		let listener = tokio::net::TcpListener::bind(addr)
			.await
			.wrap_err("Couldn't bind tcp listener to port")
			.note(format!("IPV4 Address: {addr}"))
			.map_err(Error::LiveLogging)?;
		let addr = listener
			.local_addr()
			.wrap_err("Couldn't get local addr")
			.map_err(Error::LiveLogging)?;
		debug!(%addr, "Using this port for IPC logging");

		// let cmd = async move {
		// 	loop {
		// 		trace!("CMD polled");
		// 		tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
		// 	}
		// };
		let (stop, recv) = oneshot::channel();
		let cmd = async move {
			let output = self
				.cmd([
					"-amount",
					&ParseUnits::from(amount).format_units(Unit::ETHER),
					"-vault-address",
					&vault_address.to_string(),
					"-recipient-address",
					&recipient_address.to_string(),
					"-logging-port",
					&addr.port().to_string(),
					"-gas",
					&serde_json::to_string(&gas)
						.wrap_err("Can't serialize gas")
						.map_err(Error::SerdeJson)?,
					"-data",
					&data.to_lower_hex_string(),
				])?
				.run_and_wait_for_stderr()
				.await?;
			// ignores case where recv was dropped because never used
			stop.send(()).ok();
			Result::<_, Error>::Ok(output)
		};

		let mut broadcasted_tx_hash: Option<TxHash> = None;
		let logging = logging(listener, cb, recv, &mut broadcasted_tx_hash);

		let (output, log_res) = tokio::join!(cmd, logging);
		if let Err(err) = log_res {
			error!(%err, "Error running logging task");
		}

		debug!(
			"Finished transaction {}",
			if output.is_ok() {
				"successfully"
			} else {
				"unsuccessfully"
			}
		);

		let broadcasted_tx_hash = broadcasted_tx_hash.ok_or(Error::NoBroadcastedTx)?;
		output?;
		let done = TransactionDone {
			hash: broadcasted_tx_hash,
		};

		if confirm_publish {
			let provider = alloy::providers::ProviderBuilder::new()
				.connect(self.config.broadcasting_network_rpc_node.as_str())
				.await
				.wrap_err("Couldn't connect to broadcasting network RPC node")
				.note(format!(
					"broadcasting RPC node: {}",
					self.config.broadcasting_network_rpc_node
				))
				.map_err(Error::CouldntConfirmTx)?;
			async move {
				loop {
					if let Ok(_) = provider.watch_pending_transaction(PendingTransactionConfig::new(broadcasted_tx_hash)).await {
						break;
					}
				}
			}.timeout(Duration::from_secs(15)).await.wrap_err("Couldn't find broadcasted transaction, does the tx pass account policies and are the Robos online?").map_err(Error::CouldntConfirmTx)?;
		}

		Ok(done)
	}
}
