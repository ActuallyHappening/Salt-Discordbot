
use crate::prelude::*;

	pub fn which(name: &'static str, err_msg: impl Into<String>) -> Result<Utf8PathBuf> {
		let path = ::which::which(name).map_err(|which| Error::Which {
			bin_name: name.to_owned(),
			err_msg: err_msg.into(),
			which,
		})?;
		Ok(Utf8PathBuf::try_from(path)?)
	}
