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

#[path = "tracing.rs"]
mod app_tracing;
mod chains;

pub use main::main;
mod main {
	use crate::{
		commands::admin_commands, common::GlobalState, env, prelude::*, ratelimits::RateLimits,
	};

	use tokio::sync::Notify;
	use twilight_gateway::{ConfigBuilder, Intents};
	use twilight_http::Client;
	use twilight_model::id::{Id, marker::GuildMarker};

	pub async fn main() -> Result<()> {
		let env = env::Env::default()?;
		let token = env.bot_token.clone();
		let ratelimits = RateLimits::read()?;

		info!(
			"Starting discordbot for salt public addresss {}",
			env.faucet_testnet_salt_account_address
		);

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
		let public_commands = crate::commands::public_commands();
		let application = client.current_user_application().await?.model().await?;
		let interaction_client = client.interaction(application.id);

		info!("Logged as {} with ID {}", application.name, application.id);

		interaction_client
			.set_global_commands(&public_commands)
			.await
			.wrap_err("Failed to register global public commands")?;

		// Register admin commands
		let admin_commands = crate::commands::admin_commands();
		let admin_server: Id<GuildMarker> = Id::new(1371363785985490975);
		interaction_client
			.set_guild_commands(admin_server, &admin_commands)
			.await
			.wrap_err("Couldn't set admin commands")?;

		// Start gateway shards.
		let shards =
			twilight_gateway::create_recommended(&client, config, |_id, builder| builder.build())
				.await?;
		let shard_len = shards.len();
		let mut senders = Vec::with_capacity(shard_len);
		// let mut tasks = Vec::with_capacity(shard_len);
		let mut tasks = tokio::task::JoinSet::new();
		let shutdown_now = Notify::new();
		let state = GlobalState::new(client, env, ratelimits)?;

		for shard in shards {
			senders.push(shard.sender());
			tasks.spawn(crate::runner::runner(state.clone(), shard));
		}

		tokio::signal::ctrl_c().await?;
		debug!("Ctrl-C has been registered, shutting down");
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

mod common;
pub mod env;
mod per_user_spam_filter;
mod ratelimits;
