use std::{collections::HashSet, env, path::PathBuf, sync::Arc};

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{Crate, Expr, Item, ItemKind, ModKind};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::{BytePos, FileName, SourceFile, Span};
use serde::Deserialize;

declare_lint! {
	pub FILE_MAX_LENGTH,
	Warn,
	"Rust source file exceeds the configured line limit"
}

#[derive(Clone, Copy, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Config {
	max_lines: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self { max_lines: 600 }
	}
}

pub(super) struct Pass {
	config: Config,
	seen: HashSet<BytePos>,
	out_dir: Option<PathBuf>,
}

impl_lint_pass!(Pass => [FILE_MAX_LENGTH]);

impl Pass {
	pub(super) fn new(config: Config) -> Self {
		Self {
			config,
			seen: HashSet::new(),
			out_dir: env::var_os("OUT_DIR").map(PathBuf::from),
		}
	}

	fn check_source(&mut self, cx: &EarlyContext<'_>, span: Span) {
		let synthetic: bool = span.is_dummy() || span.from_expansion();
		if synthetic {
			return;
		}
		let file: Arc<SourceFile> = cx.sess().source_map().lookup_source_file(span.lo());
		let already_checked_or_imported: bool = !self.seen.insert(file.start_pos) || file.is_imported();
		if already_checked_or_imported {
			return;
		}
		let FileName::Real(name) = &file.name else { return };
		let Some(path) = name.local_path() else { return };
		let skip_file: bool = path.extension().is_none_or(|ext| ext != "rs")
			|| self.out_dir.as_ref().is_some_and(|dir| path.starts_with(dir));
		if skip_file {
			return;
		}
		let limit: usize = self.config.max_lines;
		if let Some(&offset) = file.lines().get(limit) {
			let pos: BytePos = file.absolute_position(offset);
			let count: usize = file.count_lines();
			cx.emit_span_lint(
				FILE_MAX_LENGTH,
				Span::with_root_ctxt(pos, pos),
				DiagDecorator(|diag: &mut Diag<'_>| {
					diag.primary_message(format!("file has {count} lines; maximum is {limit}"));
				}),
			);
		}
	}
}

impl EarlyLintPass for Pass {
	fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &Crate) {
		self.check_source(cx, krate.spans.inner_span);
	}

	fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
		self.check_source(cx, item.span);
		if let ItemKind::Mod(_, _, ModKind::Loaded(_, _, spans)) = &item.kind {
			self.check_source(cx, spans.inner_span);
		}
	}

	fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
		self.check_source(cx, expr.span);
	}
}
