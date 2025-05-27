pub(crate) mod prelude {
	#![allow(unused_imports)]
	pub(crate) use std::sync::Arc;
	#[allow(unused_imports)]
	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use camino::{Utf8Path, Utf8PathBuf};

	pub(crate) use crate::errors::*;
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

	use crate::{env::Env, prelude::*};

	/// Cheap to clone
	#[derive(Clone)]
	pub struct GlobalState {
		env: Arc<Env>,
		client: Arc<Client>,
	}

	pub struct GlobalStateRef<'a> {
		pub env: &'a Env,
		pub client: &'a Client,
	}

	impl GlobalState {
		pub async fn new(client: Arc<Client>, env: Env) -> Result<Self> {
			Ok(GlobalState {
				client,
				env: Arc::new(env),
			})
		}

		pub fn get(&self) -> GlobalStateRef<'_> {
			GlobalStateRef {
				env: &self.env,
				client: &self.client,
			}
		}
	}
}

mod env {
	use url::Url;

	use crate::prelude::*;

	#[derive(serde::Deserialize)]
	#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
	pub struct Env {
		pub bot_application_id: String,
		pub bot_token: String,

		pub somnia_shannon_rpc_endpoint: Url,
		pub sepolia_arbitrum_rpc_endpoint: Url,
		pub sepolia_etherium_rpc_endpoint: Url,
		pub faucet_testnet_salt_account_address: String,

		pub private_key: String,
		pub orchestration_network_rpc_node_url: Url,
		pub salt_account_address: String,
	}

	/// Only statically includes toml if building for release,
	/// for better error messages
	/// Can refactor this is wanted
	#[allow(dead_code)]
	impl Env {
		pub(crate) fn default() -> Result<Env> {
			#[cfg(debug_assertions)]
			return Self::from_local_env();
			#[cfg(not(debug_assertions))]
			Self::from_statically_included()
		}

		fn from_local_env() -> Result<Env> {
			let path = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("env.toml");
			let file = std::fs::read_to_string(path).wrap_err("Couldn't read env.toml")?;
			let env: Env = toml::from_str(&file)
				.wrap_err("env.toml not valid toml or missing required key")?;
			Ok(env)
		}

		#[cfg(not(debug_assertions))]
		fn from_statically_included() -> Result<Env> {
			static_toml::static_toml!(
				static ENV = include_toml!("env.toml");
			);
			Ok(Env {
				bot_application_id: ENV.bot_application_id.into(),
				bot_token: ENV.bot_token.into(),
				somnia_shannon_rpc_endpoint: ENV.somnia_shannon_rpc_endpoint.parse()?,
				sepolia_arbitrum_rpc_endpoint: ENV.sepolia_arbitrum_rpc_endpoint.parse()?,
				sepolia_etherium_rpc_endpoint: ENV.sepolia_etherium_rpc_endpoint.parse()?,
				faucet_testnet_salt_account_address: ENV
					.faucet_testnet_salt_account_address
					.parse()?,
				private_key: ENV.private_key.into(),
				orchestration_network_rpc_node_url: ENV
					.orchestration_network_rpc_node_url
					.parse()?,
				salt_account_address: ENV.salt_account_address.into(),
			})
		}
	}
}

pub use main::main;
mod main {
	use crate::{common::GlobalState, env, prelude::*};

	use twilight_gateway::{ConfigBuilder, Intents};
	use twilight_http::Client;

	pub async fn main() -> Result<()> {
		let env = env::Env::default()?;
		let token = env.bot_token.clone();

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
		let state = GlobalState::new(client, env).await?;

		for shard in shards {
			senders.push(shard.sender());
			tasks.spawn(crate::runner::runner(state.clone(), shard));
		}

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
