use dylint_testing::ui::Test;

#[test]
fn package_layout() {
	Test::src_base("abos_rust_rules", "test/fixtures/tests_outside_src/src").run();
}
