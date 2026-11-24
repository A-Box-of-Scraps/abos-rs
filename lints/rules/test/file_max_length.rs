use std::fs;

use dylint_testing::ui::Test;
use tempfile::TempDir;

const LIBRARY: &str = "abos_rust_rules";

#[test]
fn default_boundary() {
	const DEFAULT_MAX_LINES: usize = 600;
	let fixtures: TempDir = tempfile::tempdir().unwrap();
	fs::write(
		fixtures.path().join("at_limit.rs"),
		format!("{}fn main() {{}}\n", "\n".repeat(DEFAULT_MAX_LINES - 1)),
	)
	.unwrap();
	fs::write(
		fixtures.path().join("over_limit.rs"),
		format!("{}fn main() {{}}", "\n".repeat(DEFAULT_MAX_LINES)),
	)
	.unwrap();
	fs::write(
		fixtures.path().join("over_limit.stderr"),
		include_str!("fixtures/file_max_length/over_limit.stderr"),
	)
	.unwrap();
	Test::src_base(LIBRARY, fixtures.path()).run();
}

#[test]
fn configured_limit_on_module_with_group_opt_in() {
	Test::src_base(LIBRARY, "test/fixtures/file_max_length/configured")
		.dylint_toml("[abos_rust_rules.file_max_length]\nmax_lines = 3")
		.rustc_flags(["-Aabos_rules", "-Wfile_max_length"])
		.run();
}

#[test]
fn rule_can_be_disabled_individually_or_as_a_group() {
	for allow in ["-Afile_max_length", "-Aabos_rules"] {
		Test::src_base(LIBRARY, "test/fixtures/file_max_length/disabled")
			.dylint_toml("[abos_rust_rules.file_max_length]\nmax_lines = 0")
			.rustc_flags(["-Dwarnings", allow])
			.run();
	}
}
