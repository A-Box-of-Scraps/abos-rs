use dylint_testing::ui::Test;

const LIBRARY: &str = "abos_rust_rules";

#[test]
fn defaults() {
	Test::src_base(LIBRARY, "tests/fixtures/no_duplicate_strings/default")
		.rustc_flags([
			"--edition=2024",
			"--cfg",
			"test",
			"-Aabos_rules",
			"-Wno_duplicate_strings",
		])
		.run();
}

#[test]
fn configured() {
	Test::src_base(LIBRARY, "tests/fixtures/no_duplicate_strings/configured")
		.dylint_toml(
			r#"[abos_rust_rules.no_duplicate_strings]
threshold = 2
shortString = 0
ignoreStrings = ["ignored"]"#,
		)
		.rustc_flags(["--edition=2024", "-Aabos_rules", "-Wno_duplicate_strings"])
		.run();
}
