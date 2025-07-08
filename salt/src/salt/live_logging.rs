use alloy_primitives::{Address, TxHash};
use serde::Deserialize;
use tokio::io::AsyncReadExt as _;

use crate::prelude::*;

/// A convenience utility type,
/// ```rust
/// use salt_sdk::Logging;
///
/// let log: Logging = Logging::from(|msg: String| tracing::info!(%msg, "Live log!"));
/// ```
pub enum LiveLogging {
	Channel(tokio::sync::mpsc::Sender<Log>),
}

#[derive(Debug, Deserialize)]
pub enum Log {
	GenericMessage(String),
	BroadcastedTx(String),
}

impl std::fmt::Display for Log {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Log::GenericMessage(msg) => write!(f, "{}", msg),
			Log::BroadcastedTx(addr) => write!(f, "Broadcasted transaction: {}", addr),
		}
	}
}

impl Log {
	pub fn from_str(msg: &str) -> Self {
		if let Ok(log) = serde_json::from_str::<serde_json::Value>(msg) {
			// parses as json
			match serde_json::from_value(log) {
				Ok(log) => log,
				Err(err) => {
					warn!(?err, "Couldn't parse valid JSON into Rust Log struct");
					Log::GenericMessage(msg.to_owned())
				}
			}
		} else {
			Log::GenericMessage(msg.to_owned())
		}
	}
}

#[test]
fn log_parses() -> color_eyre::Result<()> {
	// tracing_subscriber::fmt().init();
	let str = r#"{"BroadcastedTx":"6yWEBfXhAIJSCJTqQoIzRFpc9QC51ckbym57iH99cIYJGE5yoACAgsSIgIA="}"#;
	let log = Log::from_str(str);
	eprintln!("{:?}", log);
	assert!(matches!(log, Log::BroadcastedTx(_)));
	Ok(())
}

impl LiveLogging {
	pub fn from_cb<F>(mut cb: F) -> Self
	where
		F: FnMut(Log) + Send + 'static,
	{
		// going to do some tokio schenanigans
		let (send, mut recv) = tokio::sync::mpsc::channel(10);
		tokio::spawn(async move {
			while let Some(log) = recv.recv().await {
				cb(log);
			}
		});
		Self::Channel(send)
	}

	pub fn from_sender(sender: tokio::sync::mpsc::Sender<Log>) -> Self {
		Self::Channel(sender)
	}

	async fn send(&mut self, log: Log) {
		match self {
			// Self::Cb(cb) => cb(msg),
			Self::Channel(send) => {
				if let Err(err) = send.send(log).await {
					error!(%err, "Tried to send a live log after receiver was dropped");
				}
			}
		}
	}
}

impl From<tokio::sync::mpsc::Sender<Log>> for LiveLogging {
	fn from(value: tokio::sync::mpsc::Sender<Log>) -> Self {
		Self::Channel(value)
	}
}

#[tracing::instrument(name = "logging", skip_all)]
pub(crate) async fn logging(
	listener: tokio::net::TcpListener,
	mut logging: LiveLogging,
	mut stop_listening: tokio::sync::oneshot::Receiver<()>,
	broadcasted_tx_hash: &mut Option<TxHash>,
) -> Result<(), color_eyre::Report> {
	/// A marker for the end of a log
	/// (its a log emojie)
	const MARKER: char = '🪵';
	loop {
		trace!("Waiting for new connection");
		let mut socket = tokio::select! {
			biased;
			_ = &mut stop_listening => {
				debug!("Stopping instead of accepting another connection");
				return Ok(());
			}
			res = listener.accept() => {
				let (socket, _) = res.wrap_err("Failed to accept connection")?;
				socket
			}
		};
		debug!("Accepted a connection");

		let mut bytes: Vec<u8> = vec![];

		loop {
			let bytes_read = tokio::select! {
				biased;
				_ = &mut stop_listening => {
					debug!("Stopping instead of reading any more data");
					return Ok(());
				}
				bytes_read = socket.read_buf(&mut bytes) => {
					bytes_read.wrap_err("Couldn't read to buf")?
				}
			};
			trace!(
				"Received {} bytes, now reads: {}",
				bytes_read,
				String::from_utf8_lossy(&bytes)
			);

			let mut send = async |msg: &str| {
				let log = Log::from_str(msg);
				match &log {
					Log::BroadcastedTx(tx) => {
						*broadcasted_tx_hash = Some(TxHash::new(alloy_primitives::keccak256(tx).0));
					}
					Log::GenericMessage(_) => {
						logging.send(log).await;
					}
				}
			};

			if bytes_read == 0 {
				debug!("Disconnecting");
				if !bytes.is_empty() {
					let message = String::from_utf8_lossy(&bytes);
					trace!(%message, "Sending message with the remaining bytes because of a disconnect");
					send(&message).await;
				}
				break;
			}

			// send any messages seperated by MARKER
			let buf_clone = &bytes.clone();
			let string = String::from_utf8_lossy(buf_clone);
			let subslices: Vec<&str> = string.split(MARKER).collect();
			let (in_progress, to_send) = subslices.split_last().ok_or(eyre!("Buf empty?"))?;
			trace!("To send len: {}", to_send.len());
			// send
			for msg in to_send {
				trace!(%msg, "Sending message");
				send(&msg).await;
			}
			// replace buf
			bytes.clear();
			bytes.extend_from_slice(in_progress.as_bytes());
		}
	}
}
