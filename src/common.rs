use std::sync::Mutex;

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
	shutdown_now: Arc<Notify>,
}

#[derive(Clone, Copy)]
pub struct GlobalStateRef<'a> {
	pub client: &'a Client,
	pub env: &'a Env,
	pub ratelimits: &'a Mutex<RateLimits>,
	pub private_apis: &'a salt_private_apis::Client,
	pub per_user_spam_filters: &'a PerUserSpamFilter,
	pub shutdown_now: &'a Notify,
}

impl GlobalState {
	pub fn new(client: Arc<Client>, env: Env, ratelimits: RateLimits, shutdown_now: Notify) -> Result<Self> {
		Ok(GlobalState {
			client,
			env: Arc::new(env),
			ratelimits: Arc::new(Mutex::new(ratelimits)),
			private_apis: salt_private_apis::Client::new(),
			per_user_spam_filters: Arc::new(PerUserSpamFilter::default()),
			shutdown_now: Arc::new(shutdown_now),
		})
	}

	pub fn get(&self) -> GlobalStateRef<'_> {
		GlobalStateRef {
			env: &self.env,
			client: &self.client,
			ratelimits: &self.ratelimits,
			private_apis: &self.private_apis,
			per_user_spam_filters: &self.per_user_spam_filters,
			shutdown_now: &self.shutdown_now,
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
			shutdown_now: self.shutdown_now,
		}
	}
}
