use std::sync::{atomic::AtomicBool, Mutex};

use tokio::sync::Notify;
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
	kill_now: Arc<Notify>,
	shutting_down: Arc<AtomicBool>,
}

#[derive(Clone, Copy)]
pub struct GlobalStateRef<'a> {
	pub client: &'a Client,
	pub env: &'a Env,
	pub ratelimits: &'a Mutex<RateLimits>,
	pub private_apis: &'a salt_private_apis::Client,
	pub per_user_spam_filters: &'a PerUserSpamFilter,
	pub kill_now: &'a Notify,
	pub shutting_down: &'a AtomicBool,
}

impl GlobalState {
	pub fn new(client: Arc<Client>, env: Env, ratelimits: RateLimits, kill_now: Notify, shutting_down: Arc<AtomicBool>) -> Result<Self> {
		Ok(GlobalState {
			client,
			env: Arc::new(env),
			ratelimits: Arc::new(Mutex::new(ratelimits)),
			private_apis: salt_private_apis::Client::new(),
			per_user_spam_filters: Arc::new(PerUserSpamFilter::default()),
			kill_now: Arc::new(kill_now),
			shutting_down,
		})
	}

	pub fn get(&self) -> GlobalStateRef<'_> {
		GlobalStateRef {
			env: &self.env,
			client: &self.client,
			ratelimits: &self.ratelimits,
			private_apis: &self.private_apis,
			per_user_spam_filters: &self.per_user_spam_filters,
			kill_now: &self.kill_now,
			shutting_down: &self.shutting_down,
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
			kill_now: self.kill_now,
			shutting_down: self.shutting_down,
		}
	}
}
