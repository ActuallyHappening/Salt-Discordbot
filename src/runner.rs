use std::{
	sync::{
		LazyLock,
		atomic::{AtomicBool, Ordering},
	},
	time::Duration,
};

use crate::{common::GlobalState, prelude::*};
use tokio::sync::Notify;
use twilight_gateway::{Event, EventTypeFlags, Shard, StreamExt as _};
use twilight_model::application::interaction::InteractionData;

pub async fn runner(state: GlobalState, mut shard: Shard) {
	while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
		let event = match item {
			Ok(Event::GatewayClose(reason)) => {
				if state.get().shutting_down.load(Ordering::Acquire) {
					warn!(
						?reason,
						"Automatically closing shard because receiving a GatewayClose event {}",
						shard.id()
					);
					break;
				} else {
					warn!(
						?reason,
						"Received a GatewayClose event, but not shutting down: {}",
						shard.id()
					);
					continue;
				}
			}
			Ok(event) => event,
			Err(error) => {
				tracing::warn!(?error, "error while receiving event");
				continue;
			}
		};

		// Process Discord events
		tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
		let state = state.clone();
		tokio::spawn(async move {
			tokio::select! {
				biased;
				_ = state.get().kill_now.notified() => {
					warn!("Automatically cancelling a processing interaction because receiving a shutdown signal");
					return;
				}
				res = process_interactions(state.clone(), event).timeout(Duration::from_secs(60 * 3))
				 => {
					if let Err(err) = res {
						warn!(?err, "Timing out a processing interaction");
					}
				}
			};
		});
	}
}

/// Process incoming interactions from Discord.
pub async fn process_interactions(state: GlobalState, event: Event) {
	// We only care about interaction events.
	let mut interaction = match event {
		Event::InteractionCreate(interaction) => interaction.0,
		ignored => {
			debug!(
				"ignoring non-interaction event of kind {:?}",
				ignored.kind()
			);
			return;
		}
	};

	// Extract the command data from the interaction.
	// We use mem::take to avoid cloning the data.
	let data = match core::mem::take(&mut interaction.data) {
		Some(InteractionData::ApplicationCommand(data)) => *data,
		_ => {
			warn!("ignoring non-command interaction");
			return;
		}
	};

	if let Err(error) = crate::commands::handle_command(state, interaction, data).await {
		error!(?error, "error while handling command");
	}
}
