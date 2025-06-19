use alloy::primitives::{utils::parse_ether, U256};

pub mod rest_v5;

/// From a base 10 string encoding of a large number
pub(crate) fn u256_from_radix_wei<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: serde::Deserializer<'de>,
{
	// serde_json -F arbitrary-precision
	use serde::Deserialize as _;
	let s = serde_json::Number::deserialize(deserializer)?;
	let s = s.to_string();
	if s.contains("e+") {
		// handle 1e+22 case
		let exp_str = s
			.split("e+")
			.nth(1)
			.ok_or(serde::de::Error::custom("expected something after 'e'"))?;
		let exp: u8 = exp_str.parse().map_err(serde::de::Error::custom)?;
		let exp: U256 = exp.try_into().map_err(serde::de::Error::custom)?;
		let mantissa = exp_str
			.split("e+")
			.nth(0)
			.ok_or(serde::de::Error::custom("expected orig"))?;
		let num = U256::from_str_radix(&mantissa, 10).map_err(serde::de::Error::custom)?;
		Ok(num.pow(exp))
	} else {
		let num = U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
		Ok(num)
	}
}

/// From a base 10 string encoding of a large number
pub(crate) fn u256_from_radix_ether<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
	D: serde::Deserializer<'de>,
{
	// serde_json -F arbitrary-precision
	use serde::Deserialize as _;
	let s = serde_json::Number::deserialize(deserializer)?;
	let s = s.to_string();
	if s.contains("e+") {
		// handle 1e+22 case
		let exp_str = s
			.split("e+")
			.nth(1)
			.ok_or(serde::de::Error::custom("expected something after 'e'"))?;
		let exp: u8 = exp_str.parse().map_err(serde::de::Error::custom)?;
		let exp: U256 = exp.try_into().map_err(serde::de::Error::custom)?;
		let mantissa = exp_str
			.split("e+")
			.nth(0)
			.ok_or(serde::de::Error::custom("expected orig"))?;
		let num = U256::from_str_radix(&mantissa, 10).map_err(serde::de::Error::custom)?;
		Ok(num.pow(exp))
	} else {
		// let num = U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)?;
		let num = parse_ether(&s).map_err(serde::de::Error::custom)?;
		Ok(num)
	}
}