#[macro_export]
macro_rules! eyre_assert_eq {
	($left:expr, $right:expr $(,)?) => {
		if $left != $right {
			::color_eyre::eyre::bail!(
				"Assertion failed: {:?} ({}) != {:?} ({})",
				$left,
				stringify!($left),
				$right,
				stringify!($right)
			);
		}
	};
	($left:expr, $right:expr, $($arg:tt)+) => {
		if $left != $right {
			let string = ::std::format!($($arg)+);
			::color_eyre::eyre::bail!(
				"Assertion failed: {:?} ({}) != {:?} ({})\n{string}",
				$left,
				stringify!($left),
				$right,
				stringify!($right)
			);
		}
	};
}

pub use eyre_assert_eq;
