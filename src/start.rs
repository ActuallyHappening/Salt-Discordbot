use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
	commands::admin_commands, common::GlobalState, env, prelude::*, ratelimits::RateLimits,
};

use tokio::sync::Notify;
use twilight_gateway::{ConfigBuilder, Intents};
use twilight_http::Client;
use twilight_model::id::{Id, marker::GuildMarker};

pub async fn main() {
	let keep_restarting = Arc::new(AtomicBool::new(true));
	let shutting_down = Arc::new(AtomicBool::new(false));
	loop {
		if !keep_restarting.load(Ordering::Acquire) {
			return;
		}
		match start(keep_restarting.clone(), shutting_down.clone()).await {
			Ok(()) => {
				// ctrlc, clean exit, actually exit
				break;
			}
			Err(err) => {
				::tracing::error!(?err, "Top level error!");
				// keep looping
			}
		}
		if !keep_restarting.load(Ordering::Acquire) {
			tokio::time::sleep(std::time::Duration::from_secs(2)).await;
		}
	}
}

pub async fn start(keep_restarting: Arc<AtomicBool>, shutting_down: Arc<AtomicBool>) -> Result<()> {
	let env = env::Env::get().await?;
	let token = env.bot_token.clone();
	let ratelimits = RateLimits::read().await?;

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
	let state = GlobalState::new(
		client,
		env,
		ratelimits,
		Notify::new(),
		shutting_down.clone(),
	)?;

	for shard in shards {
		senders.push(shard.sender());
		tasks.spawn(crate::runner::runner(state.clone(), shard));
	}

	// TODO stop when no shards are left receiving
	tokio::select! {
		res = tokio::signal::ctrl_c() => {
			debug!(?res, "Ctrl-C has been registered, shutting down");
		},
		_ = state.get().kill_now.notified() => {
			debug!("Kill request has been listened to, shutting down now");
			keep_restarting.store(false, Ordering::SeqCst);
		}
	};
	shutting_down.store(true, Ordering::Release);

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
