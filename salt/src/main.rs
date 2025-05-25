mod tracing;

#[allow(unused_imports)]
use ::tracing::{trace, debug, info, warn, error};

fn main() {
	tracing::install_tracing("info,salt_sdk=trace");
	
	trace!("Started salt-sdk tracing");
}