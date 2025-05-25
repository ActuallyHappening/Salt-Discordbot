use std::sync::atomic::{AtomicBool, Ordering};

use crate::{common::GlobalState, prelude::*};
use tokio::task::JoinSet;
use twilight_gateway::{Event, EventTypeFlags, Shard, StreamExt as _};
use twilight_model::{
  application::interaction::InteractionData,
  id::{Id, marker::ChannelMarker},
};

pub static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub async fn runner(state: GlobalState, mut shard: Shard) {
  while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
    let event = match item {
      Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
      Ok(event) => event,
      Err(error) => {
        tracing::warn!(?error, "error while receiving event");
        continue;
      }
    };

    // Process Discord events
    tracing::info!(kind = ?event.kind(), shard = ?shard.id().number(), "received event");
    tokio::spawn(process_interactions(state.clone(), event));
  }
}

/// Process incoming interactions from Discord.
pub async fn process_interactions(state: GlobalState, event: Event) {
  // We only care about interaction events.
  let mut interaction = match event {
    Event::InteractionCreate(interaction) => interaction.0,
    ignored => {
      debug!(?ignored, "ignoring non-interaction event");
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
