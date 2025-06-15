use std::{collections::HashSet, sync::Mutex};

use twilight_model::id::{marker::UserMarker, Id};

use crate::prelude::*;

#[derive(Default)]
pub struct PerUserSpamFilter(Mutex<HashSet<Id<UserMarker>>>);

impl PerUserSpamFilter {
	pub fn engage(&self, discord_id: Id<UserMarker>) -> Result<Guard<'_>, PerUserErr> {
		let mut guard = self.0.lock().or_poisoned();
		if guard.contains(&discord_id) {
			Err(PerUserErr)
		} else {
			debug!(?discord_id, "Engaging per user spam filter");
			guard.insert(discord_id);
			Ok(Guard {
				mutex: self,
				discord_id,
			})
		}
	}

	fn unlock(&self, discord_id: Id<UserMarker>) {
		match self.0.lock() {
			Ok(mut lock) => {
				debug!(?discord_id, "Removing user from spam filter");
				lock.remove(&discord_id);
			}
			Err(err) => {
				error!(?discord_id, "Failed to unengage per user spam filter: Mutex is poisoned: {}", err);
			}
		}
	}

	pub(crate) fn dump(&self) -> String {
		let guard = self.0.lock().or_poisoned();
		let mut string = format!("Dump of internal per user spam filter ({}):\n", guard.len());
		if guard.is_empty() {
			string.push_str("No users are currently in the spam filter");
		} else {
			for discord_id in guard.iter() {
				string.push_str(&format!("User {discord_id} is currently limited"));
			}
		}
		string
	}

	/// Removes everybody
	pub(crate) fn purge(&self) {
		let mut guard = self.0.lock().or_poisoned();
		guard.clear();
	}
}

#[derive(Debug, thiserror::Error)]
#[error("Slow down there! One slash command per user at a time please")]
pub struct PerUserErr;

pub struct Guard<'mutex> {
	mutex: &'mutex PerUserSpamFilter,
	discord_id: Id<UserMarker>,
}

impl<'mutex> Drop for Guard<'mutex> {
	fn drop(&mut self) {
		self.mutex.unlock(self.discord_id);
	}
}
