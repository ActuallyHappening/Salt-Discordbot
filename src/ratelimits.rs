use std::collections::HashMap;

use time::{Duration, OffsetDateTime};

use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
struct ChainLimits {
	address: HashMap<Box<str>, Vec<OffsetDateTime>>,
	discord_id: HashMap<Box<str>, Vec<OffsetDateTime>>,
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
	pub address: Box<str>,
	pub discord_id: Box<str>,
	pub chain_id: u64,
	pub chain_name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
#[serde(try_from = "ser::RateLimits", into = "ser::RateLimits")]
pub struct RateLimits(HashMap<u64, ChainLimits>);

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
			.check(&key.address, &key.discord_id, &key.chain_name)
	}

	pub fn register(&mut self, key: &Key) -> Result<()> {
		if !self.0.contains_key(&key.chain_id) {
			self.0.insert(key.chain_id, ChainLimits::default());
		}
		self.0
			.get_mut(&key.chain_id)
			.unwrap()
			.register(&key.address, &key.discord_id);

		self.save()?;
		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct RateLimitErr(String);

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
	pub fn check(&mut self, address: &str, discord_id: &str, chain_name: &str) -> Result<(), RateLimitErr> {
		let address: Box<str> = address.to_owned().into_boxed_str();
		let discord_id = discord_id.to_owned().into_boxed_str();
		let now = OffsetDateTime::now_utc();

		if !self.address.contains_key(&address) {
			self.address.insert(address.clone(), Vec::new());
		}
		let address_valid = Self::address_valid(&now, self.address.get(&address).unwrap());

		if !self.discord_id.contains_key(&discord_id) {
			self.discord_id.insert(discord_id.clone(), Vec::new());
		}
		let discord_valid = Self::discord_id_valid(&now, self.discord_id.get(&discord_id).unwrap());

		match (address_valid, discord_valid) {
			(true, false) => Err(RateLimitErr(format!(
				"You've reached your limit for fauceting for your discord account on {chain_name}. Please try again in 24 hours.",
			))),
			(false, true) => Err(RateLimitErr(format!(
				"You've reached your limit for fauceting to this wallet address on {chain_name}. Please try again in 24 hours.",
			))),
			(false, false) => Err(RateLimitErr(format!(
				"You've reached your limit for fauceting to this wallet address *and* your discord account on {chain_name}. Please try again in 24 hours.",
			))),
			(true, true) => Ok(()),
		}
	}

	/// Automatically saves
	pub fn register(&mut self, address: &str, discord_id: &str) {
		let address: Box<str> = address.to_owned().into_boxed_str();
		let discord_id = discord_id.to_owned().into_boxed_str();
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

	fn address_valid(now: &OffsetDateTime, previous: &Vec<OffsetDateTime>) -> bool {
		// 2 per day
		let range = Duration::DAY;
		let max_num_in_range = 2;

		let previous_num = previous
			.iter()
			.filter(|time| (**time - *now) < range)
			.count();

		if previous_num >= max_num_in_range {
			false
		} else {
			true
		}
	}

	fn discord_id_valid(now: &OffsetDateTime, previous: &Vec<OffsetDateTime>) -> bool {
		// 3 per day
		let range = Duration::DAY;
		let max_num_in_range = 3;

		let previous_num = previous
			.iter()
			.filter(|time| (**time - *now) < range)
			.count();

		if previous_num >= max_num_in_range {
			false
		} else {
			true
		}
	}
}
