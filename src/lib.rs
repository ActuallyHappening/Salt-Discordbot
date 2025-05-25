pub(crate) mod prelude {
	pub(crate) use std::sync::Arc;
	#[allow(unused_imports)]
	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use crate::errors::*;
	pub(crate) use db::prelude::*;
}

pub(crate) mod errors {
	#![allow(unused_imports)]

	pub use color_eyre::Result;
	pub use color_eyre::eyre::{WrapErr as _, bail, eyre};
}

pub mod commands;
mod runner;
pub mod tracing;
mod presence {
	use twilight_model::gateway::{
		payload::outgoing::update_presence::UpdatePresencePayload,
		presence::{ActivityType, MinimalActivity, Status},
	};

	pub fn presence() -> UpdatePresencePayload {
		let activity = MinimalActivity {
			kind: ActivityType::Watching,
			name: String::from("Salt bot (presence)"),
			url: None,
		};

		UpdatePresencePayload {
			activities: vec![activity.into()],
			afk: false,
			since: None,
			status: Status::Online,
		}
	}
}

mod common {
	use twilight_http::Client;

	use crate::prelude::*;

	/// Cheap to clone
	#[derive(Clone)]
	pub struct GlobalState {
		salt: Salt,
		client: Arc<Client>,
	}

	pub struct GlobalStateRef<'a> {
		pub db: Db<auth::Root>,
		pub client: &'a Client,
	}

	impl GlobalState {
		pub async fn new(client: Arc<Client>) -> Result<Self> {
			let salt = Salt::new();
			Ok(GlobalState { client, salt })
		}

		pub fn get(&self) -> GlobalStateRef<'_> {
			GlobalStateRef {
				db: self.db.clone(),
				client: &self.client,
			}
		}
	}

	pub fn to_string_pretty<T: serde::Serialize>(value: &T) -> String {
		serde_json::to_string_pretty(value)
			.wrap_err("Couldn't JSONify value")
			.unwrap_or_else(|err| err.to_string())
	}
}

mod salt {
	//! link to salt jsr library integration

	pub struct Salt {
		path: Utf8PathBuf,
	}

	impl Salt {
		pub fn new() -> color_eyre::Result<Salt> {
			todo!()
		}
	}
}

pub use main::main;
mod main {
	use crate::{common::GlobalState, prelude::*};

	use twilight_gateway::{ConfigBuilder, Intents};
	use twilight_http::Client;

	pub async fn main() -> Result<()> {
		let token = ::env::discord::SECRET_API_KEY.to_string();

		// Initialize Twilight HTTP client and gateway configuration.
		let client = Arc::new(Client::new(token.clone()));
		let config = ConfigBuilder::new(token.clone(), Intents::GUILD_MESSAGES)
			.presence(crate::presence::presence())
			.build();

		// Register global commands.
		let commands = crate::commands::commands();
		let application = client.current_user_application().await?.model().await?;
		let interaction_client = client.interaction(application.id);

		info!("Logged as {} with ID {}", application.name, application.id);

		if let Err(error) = interaction_client.set_global_commands(&commands).await {
			tracing::error!(?error, "failed to register commands");
		}

		// Start gateway shards.
		let shards =
			twilight_gateway::create_recommended(&client, config, |_id, builder| builder.build())
				.await?;
		let shard_len = shards.len();
		let mut senders = Vec::with_capacity(shard_len);
		// let mut tasks = Vec::with_capacity(shard_len);
		let mut tasks = tokio::task::JoinSet::new();
		let state = GlobalState::new(client).await?;

		for shard in shards {
			senders.push(shard.sender());
			tasks.spawn(crate::runner::runner(state.clone(), shard));
		}

		tasks.spawn(crate::runner::logging_runner(state));

		tokio::signal::ctrl_c().await?;
		crate::runner::SHUTDOWN.store(true, std::sync::atomic::Ordering::Relaxed);
		for sender in senders {
			// Ignore error if shard's already shutdown.
			_ = sender.close(twilight_gateway::CloseFrame::NORMAL);
		}

		// for jh in tasks {
		//   _ = jh.await;
		// }
		// for res in tasks.join_all().await {
		//   res?;
		// }
		tasks.join_all().await;

		Ok(())
	}
}
