#![allow(unused_imports)]

pub(crate) mod prelude {
	pub(crate) use std::sync::Arc;
	pub(crate) use ystd::prelude::*;

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

pub use start::main;
mod start;

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
