use std::fs;

use dylint_testing::ui::Test;
use tempfile::TempDir;

#[test]
fn binding_types() {
	Test::src_base("abos_rust_rules", "test/fixtures/explicit_binding_types").run();
}

#[test]
fn explicit_type_ascription() {
	// Only rustc should parse this fixture: it uses compiler-internal syntax.
	let fixtures: TempDir = tempfile::tempdir().unwrap();
	fs::write(
		fixtures.path().join("type_ascription.rs"),
		include_str!("fixtures/explicit_binding_types/type_ascription.rs.in"),
	)
	.unwrap();
	Test::src_base("abos_rust_rules", fixtures.path()).run();
}
