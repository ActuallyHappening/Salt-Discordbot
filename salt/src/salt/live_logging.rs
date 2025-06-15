use tokio::io::AsyncReadExt as _;

use crate::prelude::*;

/// A convenience utility type,
/// ```rust
/// use salt_sdk::Logging;
///
/// let log: Logging = Logging::from(|msg: String| tracing::info!(%msg, "Live log!"));
/// ```
pub enum LiveLogging {
	Channel(tokio::sync::mpsc::Sender<String>),
}

impl LiveLogging {
	pub fn from_cb<F>(mut cb: F) -> Self
	where
		F: FnMut(String) + Send + 'static,
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

	pub fn from_sender(sender: tokio::sync::mpsc::Sender<String>) -> Self {
		Self::Channel(sender)
	}

	async fn send(&mut self, msg: String) {
		match self {
			// Self::Cb(cb) => cb(msg),
			Self::Channel(send) => {
				if let Err(err) = send.send(msg).await {
					error!(%err, "Tried to send a live log after receiver was dropped");
				}
			}
		}
	}
}

impl From<tokio::sync::mpsc::Sender<String>> for LiveLogging {
	fn from(value: tokio::sync::mpsc::Sender<String>) -> Self {
		Self::Channel(value)
	}
}

#[tracing::instrument(name = "logging", skip_all)]
pub(crate) async fn logging(
	listener: tokio::net::TcpListener,
	mut logging: LiveLogging,
	mut stop_listening: tokio::sync::oneshot::Receiver<()>,
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

			if bytes_read == 0 {
				debug!("Disconnecting");
				if !bytes.is_empty() {
					let message = String::from_utf8_lossy(&bytes);
					trace!(%message, "Sending message with the remaining bytes because of a disconnect");
					logging.send(message.into_owned()).await;
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
				logging.send(msg.to_string()).await;
			}
			// replace buf
			bytes.clear();
			bytes.extend_from_slice(in_progress.as_bytes());
		}
	}
}