#![allow(unused_imports)]

pub(crate) mod prelude {
	pub(crate) use std::sync::Arc;
	#[allow(unused_imports)]
	pub(crate) use tracing::{debug, error, info, trace, warn};

	pub(crate) use camino::{Utf8Path, Utf8PathBuf};
	pub(crate) use or_poisoned::OrPoisoned as _;

	pub(crate) use crate::errors::*;
}

pub(crate) mod errors {
	#![allow(unused_imports)]

	pub use color_eyre::Result;
	pub use color_eyre::eyre::{WrapErr as _, bail, eyre};
}

mod chains;

pub use main::main;
mod main {
	use crate::{common::GlobalState, env, prelude::*, ratelimits::RateLimits};

	use twilight_gateway::{ConfigBuilder, Intents};
	use twilight_http::Client;

	pub async fn main() -> Result<()> {
		let env = env::Env::default()?;
		let token = env.bot_token.clone();
		let ratelimits = RateLimits::read()?;
		
		info!("Starting discordbot for salt public addresss {}", env.faucet_testnet_salt_account_address);

		// Initialize Twilight HTTP client and gateway configuration.
		let client = Arc::new(Client::new(token.clone()));
		let config = ConfigBuilder::new(token.clone(), Intents::GUILD_MESSAGES)
			.presence(crate::presence::presence())
			.identify_properties(
				twilight_model::gateway::payload::outgoing::identify::IdentifyProperties {
					browser: "twilight.rs".to_owned(),
					device: if cfg!(debug_assertions) {
						"Caleb's personal PC"
					} else {
						"Salt-managed server"
					}
					.to_owned(),
					os: std::env::consts::OS.to_owned(),
				},
			)
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
		let state = GlobalState::new(client, env, ratelimits)?;

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
			kind: ActivityType::Listening,
			name: String::from("blockchain transactions in the ether ..."),
			url: Some("https://github.com/ActuallyHappening/Salt-Discordbot".into()),
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
	use std::sync::Mutex;

	use twilight_http::Client;

	use crate::{
		env::Env, per_user_spam_filter::PerUserSpamFilter, prelude::*, ratelimits::RateLimits,
	};

	/// Cheap to clone
	#[derive(Clone)]
	pub struct GlobalState {
		client: Arc<Client>,
		env: Arc<Env>,
		ratelimits: Arc<Mutex<RateLimits>>,
		private_apis: salt_private_apis::Client,
		per_user_spam_filters: Arc<PerUserSpamFilter>,
	}

	#[derive(Clone, Copy)]
	pub struct GlobalStateRef<'a> {
		pub client: &'a Client,
		pub env: &'a Env,
		pub ratelimits: &'a Mutex<RateLimits>,
		pub private_apis: &'a salt_private_apis::Client,
		pub per_user_spam_filters: &'a PerUserSpamFilter,
	}

	impl GlobalState {
		pub fn new(client: Arc<Client>, env: Env, ratelimits: RateLimits) -> Result<Self> {
			Ok(GlobalState {
				client,
				env: Arc::new(env),
				ratelimits: Arc::new(Mutex::new(ratelimits)),
				private_apis: salt_private_apis::Client::new(),
				per_user_spam_filters: Arc::new(PerUserSpamFilter::default()),
			})
		}

		pub fn get(&self) -> GlobalStateRef<'_> {
			GlobalStateRef {
				env: &self.env,
				client: &self.client,
				ratelimits: &self.ratelimits,
				private_apis: &self.private_apis,
				per_user_spam_filters: &self.per_user_spam_filters,
			}
		}
	}

	impl<'a> GlobalStateRef<'a> {
		pub fn reborrow(&self) -> GlobalStateRef<'_> {
			GlobalStateRef {
				env: self.env,
				client: self.client,
				ratelimits: self.ratelimits,
				private_apis: self.private_apis,
				per_user_spam_filters: self.per_user_spam_filters,
			}
		}
	}
}

pub mod env;
mod ratelimits;
mod per_user_spam_filter {
	use std::{collections::HashSet, sync::Mutex};

	use crate::prelude::*;

	#[derive(Default)]
	pub struct PerUserSpamFilter(Mutex<HashSet<u64>>);

	impl PerUserSpamFilter {
		pub fn engage(&self, discord_id: u64) -> Result<Guard<'_>, PerUserErr> {
			let mut guard = self.0.lock().or_poisoned();
			if guard.contains(&discord_id) {
				Err(PerUserErr)
			} else {
				guard.insert(discord_id);
				Ok(Guard {
					mutex: self,
					discord_id,
				})
			}
		}
	}

	#[derive(Debug, thiserror::Error)]
	#[error("Slow down there! One slash command per user at a time please")]
	pub struct PerUserErr;

	pub struct Guard<'mutex> {
		mutex: &'mutex PerUserSpamFilter,
		discord_id: u64,
	}

	impl<'mutex> Drop for Guard<'mutex> {
		fn drop(&mut self) {
			match self.mutex.0.lock() {
				Ok(mut lock) => {
					debug!("Removing user from spam filter becuase the guard dropped");
					lock.remove(&self.discord_id);
				}
				Err(_) => {
					error!(?self.discord_id, "Failed to unengage per user spam filter");
				}
			}
		}
	}
}
