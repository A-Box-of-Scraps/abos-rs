use std::collections::HashSet;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(try_from = "Options")]
pub(crate) struct Config {
	pub(super) threshold: usize,
	pub(super) short_string: usize,
	pub(super) ignore_strings: HashSet<String>,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
	threshold: usize,
	short_string: usize,
	ignore_strings: Vec<String>,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			threshold: 3,
			short_string: 10,
			ignore_strings: Vec::new(),
		}
	}
}

impl TryFrom<Options> for Config {
	type Error = &'static str;

	fn try_from(options: Options) -> Result<Self, Self::Error> {
		const MIN_THRESHOLD: usize = 2;
		if options.threshold < MIN_THRESHOLD {
			return Err("threshold must be at least 2");
		}
		let entries: usize = options.ignore_strings.len();
		let ignore_strings: HashSet<String> = options.ignore_strings.into_iter().collect();
		if ignore_strings.len() != entries {
			return Err("ignoreStrings must contain unique strings");
		}
		Ok(Self {
			threshold: options.threshold,
			short_string: options.short_string,
			ignore_strings,
		})
	}
}

impl Default for Config {
	fn default() -> Self {
		Options::default().try_into().unwrap()
	}
}
