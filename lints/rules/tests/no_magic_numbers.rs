use dylint_testing::ui::Test;

#[test]
fn no_magic_numbers() {
	Test::src_base("abos_rust_rules", "tests/fixtures/no_magic_numbers").run();
}
