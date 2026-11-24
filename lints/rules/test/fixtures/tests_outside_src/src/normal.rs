// rustc-env:CARGO_MANIFEST_DIR=test/fixtures/tests_outside_src

#[cfg(test)]
fn fixture() {}

#[cfg(all(false, test))]
fn never() {}

#[allow(tests_outside_src, reason = "exercise individual opt-out")]
#[cfg(test)]
fn allowed_fixture() {}

fn main() {}
