use dylint_testing::ui::Test;

#[test]
fn package_layout() {
	Test::src_base("abos_rust_rules", "tests/fixtures/tests_outside_src/src").run();
}
