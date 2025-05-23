use color_eyre::eyre::{WrapErr as _, eyre};
use toml_edit::{Item, Value};

fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;

	let env_toml = std::fs::read_to_string("env.toml").wrap_err("Couldn't read env.toml")?;
	let env_toml: toml::Table =
		toml::from_str(&env_toml).wrap_err("Couldn't parse env.toml as toml")?;

	let sample =
		std::fs::read_to_string("env.sample.toml").wrap_err("Couldn't read env.sample.toml")?;
	let mut sample: toml_edit::DocumentMut =
		sample.parse().wrap_err("Couldn't parse env.sample.toml")?;

	let mut errors = Vec::new();
	let sentinal = "document me please";
	for (key, _value) in env_toml {
		if sample.get(&key).is_none() {
			sample[&key] = sentinal.into();
			errors.push(eyre!("Missing key {} in env.sample.toml", key));
		} else {
			if let Item::Value(Value::String(str)) = &sample[&key] {
				if str.value() == &sentinal.to_owned() {
					errors.push(eyre!("Please document key {} in env.sample.toml", key));
				}
			}
		}
	}

	sample.fmt();
	std::fs::write("env.sample.toml", sample.to_string())
		.wrap_err("Couldn't write back out to env.sample.toml")?;

	if errors.len() > 0 {
		for err in errors {
			println!("cargo::error={}", err);
		}
	}

	Ok(())
}
