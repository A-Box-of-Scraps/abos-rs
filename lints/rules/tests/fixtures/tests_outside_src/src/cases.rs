// rustc-env:CARGO_MANIFEST_DIR=tests/fixtures/tests_outside_src
// compile-flags: --test
// aux-build: framework.rs
#![allow(dead_code)]

extern crate framework;

#[cfg(all(test, windows))]
fn windows_fixture() {}

#[cfg_attr(not(test), cfg(test))]
fn fixture() {}

#[cfg(any(test, windows))]
fn production_on_windows() {}

#[cfg(not(test))]
fn production_only() {}

#[cfg_attr(test, derive(Debug))]
struct Token;

#[cfg(test)]
mod inline {
    #[test]
    fn nested() {}
}

#[cfg(test)]
#[cfg_attr(test, path = "../unit/src/private.rs")]
mod private;

#[test]
fn direct() {}

#[cfg_attr(test, test)]
fn conditional() {}

#[framework::check]
fn framework_test() {}

fn parse() {
    #[cfg(test)]
    fixture();
    if cfg!(all(test, windows)) {
        let _ = Token;
    }
    if cfg!(not(test)) {} else {
        let _ = Token;
    }
}
