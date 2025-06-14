use std::collections::HashMap;

use alloy::primitives::Address;
use std::time::Duration;
use time::OffsetDateTime;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(try_from = "ser::RateLimits", into = "ser::RateLimits")]
pub struct RateLimits(HashMap<u64, ChainLimits>);

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
struct ChainLimits {
	address: HashMap<Address, Vec<OffsetDateTime>>,
	discord_id: HashMap<Id<UserMarker>, Vec<OffsetDateTime>>,
}

#[test]
fn chain_limits_serde() {
	let toml = r##"
		123 = { address = {}, discord_id = {}}
		"##;
	let _: RateLimits = toml::from_str(toml).expect("to deserialize");

	let toml = r##"
		"##;
	let _: RateLimits = toml::from_str(toml).expect("to deserialize");

	RateLimits::read().expect("to deserialize");
}

pub struct Key {
	pub address: Address,
	pub discord_id: Id<UserMarker>,
	pub chain_id: u64,
	pub chain_name: &'static str,
}

/// Doesn't use u64 as key
mod ser {
	use std::collections::HashMap;

	use serde::{Deserialize, Serialize};

	use crate::prelude::*;

	#[derive(Serialize, Deserialize)]
	pub(crate) struct RateLimits(HashMap<String, super::ChainLimits>);

	impl TryFrom<RateLimits> for super::RateLimits {
		type Error = color_eyre::Report;
		fn try_from(value: RateLimits) -> Result<Self, Self::Error> {
			value
				.0
				.into_iter()
				.map(|(k, v)| {
					let k: u64 = k.parse().wrap_err("Invalid number key")?;
					Result::<_, Self::Error>::Ok((k, v))
				})
				.collect::<Result<_, Self::Error>>()
				.map(Self)
		}
	}

	impl From<super::RateLimits> for RateLimits {
		fn from(value: super::RateLimits) -> Self {
			Self(
				value
					.0
					.into_iter()
					.map(|(k, v)| (k.to_string(), v.into()))
					.collect(),
			)
		}
	}
}

impl RateLimits {
	pub fn check(&mut self, key: &Key) -> Result<(), RateLimitErr> {
		if !self.0.contains_key(&key.chain_id) {
			self.0.insert(key.chain_id, ChainLimits::default());
		}
		self.0
			.get_mut(&key.chain_id)
			.unwrap()
			.check(key.address, key.discord_id, &key.chain_name)
	}

	fn describe(&mut self, discord_id: Id<UserMarker>) -> String {
		let now = OffsetDateTime::now_utc();
		let mut ret = String::new();
		for (chain_id, chain_limits) in &mut self.0 {
			ret.push_str(&format!("Chain ID: {}\n", chain_id));
			ret.push_str(&chain_limits.describe(&now, discord_id));
			ret.push('\n');
		}
		ret
	}

	pub fn register(&mut self, key: &Key) -> Result<()> {
		if !self.0.contains_key(&key.chain_id) {
			self.0.insert(key.chain_id, ChainLimits::default());
		}
		self.0
			.get_mut(&key.chain_id)
			.unwrap()
			.register(key.address, key.discord_id);

		self.save()?;
		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct RateLimitErr(String);

fn format_date(date: OffsetDateTime) -> String {
	let format = time::format_description::parse(
		"[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour \
        sign:mandatory]:[offset_minute]:[offset_second]",
	)
	.unwrap();
	date.format(&format).unwrap()
}

/// Simple toml file storage
impl RateLimits {
	pub fn read() -> Result<Self> {
		let path = Utf8PathBuf::from("ratelimits.toml");
		let file = std::fs::read_to_string(path)?;
		let data: Self = toml::from_str(&file)?;
		Ok(data)
	}

	pub fn save(&self) -> Result<()> {
		let path = Utf8PathBuf::from("ratelimits.toml");
		let data = toml::to_string(&self)?;
		std::fs::write(path, data)?;
		Ok(())
	}
}

impl ChainLimits {
	pub fn check(
		&mut self,
		address: Address,
		discord_id: Id<UserMarker>,
		chain_name: &'static str,
	) -> Result<(), RateLimitErr> {
		let now = OffsetDateTime::now_utc();

		if !self.address.contains_key(&address) {
			self.address.insert(address.clone(), Vec::new());
		}
		let address_valid = Self::address_valid(&now, self.address.get(&address).unwrap());

		if !self.discord_id.contains_key(&discord_id) {
			self.discord_id.insert(discord_id.clone(), Vec::new());
		}
		let discord_valid = Self::discord_id_valid(&now, self.discord_id.get(&discord_id).unwrap());

		let format_duration = |duration: time::Duration| {
			let hours = duration.whole_hours();
			let h_plural = if hours == 1 { "" } else { "s" };
			let minutes = duration.whole_minutes() % 60;
			let m_plural = if minutes == 1 { "" } else { "s" };
			let seconds = duration.whole_seconds() % 60;
			let s_plural = if seconds == 1 { "" } else { "s" };
			format!(
				"{hours} hour{h_plural} {minutes} minute{m_plural} (and {seconds} second{s_plural})"
			)
		};

		match (address_valid, discord_valid) {
			(Ok(_), Err((diff, _))) => Err(RateLimitErr(format!(
				"You've reached your limit for fauceting for your discord account on {chain_name}. Please try again in {}.",
				format_duration(diff),
			))),
			(Err((diff, _)), Ok(_)) => Err(RateLimitErr(format!(
				"You've reached your limit for fauceting to this wallet address on {chain_name}. Please try again in {}.",
				format_duration(diff),
			))),
			(Err((diff1, _)), Err((diff2, _))) => Err(RateLimitErr(format!(
				"Impressive! You've reached your limit for fauceting to this wallet address and your discord account on {chain_name}. Please try again in {}.",
				format_duration(diff1.max(diff2)),
			))),
			(Ok(_), Ok(_)) => Ok(()),
		}
	}

	fn describe(&mut self, now: &OffsetDateTime, discord_id: Id<UserMarker>) -> String {
		let mut ret = String::new();
		for (address, records) in self.address.clone() {
			let records = records
				.into_iter()
				.map(|date| format_date(date.clone()))
				.collect::<Vec<String>>();
			ret.push_str(&format!(
				"Address (ratelimited: {:?}): {:?}",
				self.check(address, discord_id, "").is_ok(),
				records
			));
		}
		ret.push_str("\n");
		ret
	}

	/// Automatically saves
	pub fn register(&mut self, address: Address, discord_id: Id<UserMarker>) {
		let now = OffsetDateTime::now_utc();

		if !self.address.contains_key(&address) {
			self.address.insert(address.clone(), Vec::new());
		}
		self.address.get_mut(&address).unwrap().push(now);

		if !self.discord_id.contains_key(&discord_id) {
			self.discord_id.insert(discord_id.clone(), Vec::new());
		}
		self.discord_id.get_mut(&discord_id).unwrap().push(now);
	}

	/// Return Ok(_) or Err(_) with the relevant datetimes under consideration
	fn address_valid(
		now: &OffsetDateTime,
		previous: &[OffsetDateTime],
	) -> Result<Vec<OffsetDateTime>, (time::Duration, Vec<OffsetDateTime>)> {
		// 2 per day
		let range = time::Duration::DAY;
		let max_num_in_range = 2;

		let previous_num = previous
			.iter()
			.filter(|time| **time <= *now)
			.filter(|time| (*now - **time) < range);

		if previous_num.clone().count() >= max_num_in_range {
			let earliest = previous_num.clone().min().unwrap();
			let diff = range - (*now - *earliest).abs();
			Err((diff, previous_num.cloned().collect()))
		} else {
			Ok(previous_num.cloned().collect())
		}
	}

	/// Return Ok(_) or Err(_) with the relevant datetimes under consideration
	fn discord_id_valid(
		now: &OffsetDateTime,
		previous: &Vec<OffsetDateTime>,
	) -> Result<Vec<OffsetDateTime>, (time::Duration, Vec<OffsetDateTime>)> {
		// 3 per day
		let range = time::Duration::DAY;
		let max_num_in_range = 3;

		let previous_num = previous
			.iter()
			.filter(|time| **time <= *now)
			.filter(|time| (*now - **time) < range);

		if previous_num.clone().count() >= max_num_in_range {
			let earliest = previous_num.clone().min().unwrap();
			let diff = range - (*now - *earliest).abs();
			Err((diff, previous_num.cloned().collect()))
		} else {
			Ok(previous_num.cloned().collect())
		}
	}
}
