#[macro_export]
macro_rules! eyre_assert_eq {
	($left:expr, $right:expr) => {
		if $left != $right {
			::color_eyre::eyre::bail!("Assertion failed: {:?} != {:?}", $left, $right);
		}
	};
}

pub use eyre_assert_eq;
