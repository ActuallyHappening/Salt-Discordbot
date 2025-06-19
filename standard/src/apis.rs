use alloy::primitives::{U256, utils::parse_ether};

// pub mod rest_v4;
pub mod rest_v5;

const RPC_URL: &str = "https://dream-rpc.somnia.network/";

pub trait CheckInvariants {
	async fn check_invariants(&self) -> color_eyre::Result<()>;
}

// Generate the contract bindings for the ERC20 interface.
alloy::sol! {
	// The `rpc` attribute enables contract interaction via the provider.
	#[sol(rpc, abi)]
	contract ERC20 {
		function name() public view returns (string);
		function symbol() public view returns (string);
		function decimals() public view returns (uint8);
		function totalSupply() public view returns (uint256);
		function balanceOf(address account) public view returns (uint256);
		function transfer(address recipient, uint256 amount) public returns (bool);
		function allowance(address owner, address spender) public view returns (uint256);
		function approve(address spender, uint256 amount) public returns (bool);
		function transferFrom(address sender, address recipient, uint256 amount) public returns (bool);

		event Transfer(address indexed from, address indexed to, uint256 value);
		event Approval(address indexed owner, address indexed spender, uint256 value);
	}
}

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

pub use string_or_none::lazy_empty_str;
mod string_or_none {
	use std::{marker::PhantomData, str::FromStr};

	use serde::Deserialize as _;

	pub fn lazy_empty_str<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
	where
		D: serde::Deserializer<'de>,
		T: FromStr,
		<T as FromStr>::Err: std::fmt::Display,
	{
		deserializer.deserialize_any(StringOrNone(PhantomData))
	}

	#[test]
	fn string_or_none() -> color_eyre::Result<()> {
		crate::app_tracing::install_tracing("info").ok();

		let json = serde_json::json!({ "example": null });
		#[derive(serde::Deserialize)]
		struct Example {
			#[serde(deserialize_with = "lazy_empty_str")]
			example: Option<String>,
		}
		let data: Example = serde_json::from_value(json)?;
		Ok(())
	}

	#[derive(Default)]
	struct StringOrNone<T>(PhantomData<fn() -> T>);

	impl<'de, T> serde::de::Visitor<'de> for StringOrNone<T>
	where
		T: FromStr,
		<T as FromStr>::Err: std::fmt::Display,
	{
		type Value = Option<T>;

		fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
			formatter.write_str(&format!(
				"a string ({}) or null/undefined",
				std::any::type_name::<T>()
			))
		}

		fn visit_none<E>(self) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(None)
		}

		fn visit_unit<E>(self) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			Ok(None)
		}

		fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
		where
			D: serde::Deserializer<'de>,
		{
			let s = String::deserialize(deserializer)?;
			if s.is_empty() {
				Ok(None)
			} else {
				Ok(Some(s.parse().map_err(serde::de::Error::custom)?))
			}
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: serde::de::Error,
		{
			if v.is_empty() {
				Ok(None)
			} else {
				Ok(Some(v.parse().map_err(serde::de::Error::custom)?))
			}
		}
	}
}
