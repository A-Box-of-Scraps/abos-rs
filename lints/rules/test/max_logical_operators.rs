use dylint_testing::ui::Test;

const LIBRARY: &str = "abos_rust_rules";

#[test]
fn default_limit() {
	Test::src_base(LIBRARY, "test/fixtures/max_logical_operators/default")
		.rustc_flags(["--edition=2024", "-Aabos_rules", "-Wmax_logical_operators"])
		.run();
}

#[test]
fn zero_limit() {
	Test::src_base(LIBRARY, "test/fixtures/max_logical_operators/zero")
		.dylint_toml("[abos_rust_rules.max_logical_operators]\nmax = 0")
		.rustc_flags(["-Aabos_rules", "-Wmax_logical_operators"])
		.run();
}
