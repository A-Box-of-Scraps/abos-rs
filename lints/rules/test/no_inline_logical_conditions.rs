use dylint_testing::ui::Test;

#[test]
fn conditions() {
	Test::src_base("abos_rust_rules", "test/fixtures/no_inline_logical_conditions")
		.rustc_flags(["--edition=2024", "-Aabos_rules", "-Wno_inline_logical_conditions"])
		.run();
}
