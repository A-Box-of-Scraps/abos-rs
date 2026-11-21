#![feature(default_field_values, rustc_private)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod explicit_binding_types;
mod file_max_length;
mod no_magic_numbers;
mod tests_outside_src;

use rustc_lint::LintStore;
use rustc_lint_defs::{Lint, LintId};
use rustc_session::Session;
use serde::Deserialize;

dylint_linting::dylint_library!();

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
	file_max_length: file_max_length::Config,
}

#[allow(unsafe_code, reason = "Dylint resolves this entry point by symbol name")]
#[unsafe(no_mangle)]
pub fn register_lints(session: &Session, store: &mut LintStore) {
	dylint_linting::init_config(session);
	let config: Config = dylint_linting::config_or_default("abos_rust_rules");
	let rules: &[&Lint] = &[
		explicit_binding_types::EXPLICIT_BINDING_TYPES,
		file_max_length::FILE_MAX_LENGTH,
		no_magic_numbers::NO_MAGIC_NUMBERS,
		tests_outside_src::TESTS_OUTSIDE_SRC,
	];
	store.register_lints(rules);
	store.register_group(
		true,
		"abos_rules",
		None,
		rules.iter().map(|rule| LintId::of(rule)).collect(),
	);
	store.register_early_lint_pass(Box::new(|| Box::new(no_magic_numbers::Pass::default())));
	tests_outside_src::register(store);
	explicit_binding_types::register(store);
	store.register_early_lint_pass(Box::new(move || {
		Box::new(file_max_length::Pass::new(config.file_max_length))
	}));
}
