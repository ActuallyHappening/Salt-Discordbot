use std::collections::HashMap;

use time::{Duration, OffsetDateTime};

use crate::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct RateLimits {
	address: HashMap<Box<str>, Vec<OffsetDateTime>>,
	discord_id: HashMap<Box<str>, Vec<OffsetDateTime>>,
}

pub enum RateLimit {
	Ratelimited { msg: String },
	Ok,
}

impl RateLimit {
	pub fn is_ok(&self) -> bool {
		matches!(self, RateLimit::Ok)
	}
}

/// Simple toml file storage
impl RateLimits {
	pub fn read() -> Result<Self> {
		let path = Utf8PathBuf::from("ratelimits.toml");
		let file = std::fs::read_to_string(path)?;
		let data: RateLimits = toml::from_str(&file)?;
		Ok(data)
	}

	pub fn save(&self) -> Result<()> {
		let path = Utf8PathBuf::from("ratelimits.toml");
		let data = toml::to_string(&self)?;
		std::fs::write(path, data)?;
		Ok(())
	}
}

impl RateLimits {
	pub fn check(&mut self, address: &str, discord_id: &str) -> RateLimit {
		let address: Box<str> = address.to_owned().into_boxed_str();
		let discord_id = discord_id.to_owned().into_boxed_str();
		let now = OffsetDateTime::now_utc();

		if !self.address.contains_key(&address) {
			self.address.insert(address.clone(), Vec::new());
		}
		let address_valid = !Self::address_valid(&now, self.address.get(&address).unwrap());

		if !self.discord_id.contains_key(&discord_id) {
			self.discord_id.insert(discord_id.clone(), Vec::new());
		}
		let discord_valid =
			!Self::discord_id_valid(&now, self.discord_id.get(&discord_id).unwrap());

		match (address_valid, discord_valid) {
			(true, false) => RateLimit::Ratelimited {
				msg: String::from("Too many requests from this discord account"),
			},
			(false, true) => RateLimit::Ratelimited {
				msg: String::from("Too many requests for this wallet address"),
			},
			(false, false) => RateLimit::Ratelimited {
				msg: String::from(
					"Too many requests from this discord account and wallet address (impressive)",
				),
			},
			(true, true) => RateLimit::Ok,
		}
	}

	/// Automatically saves
	pub fn register(&mut self, address: &str, discord_id: &str) -> Result<()> {
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

		self.save()?;
		Ok(())
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
