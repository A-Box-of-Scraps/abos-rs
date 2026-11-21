mod config;
mod source;

use std::{
	collections::BTreeMap,
	mem,
	sync::{Arc, Mutex},
};

pub(super) use config::Config;
use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::LitKind;
use rustc_errors::{Diag, DiagDecorator};
use rustc_hir::{Body, Expr, HirId, Lit, def::DefKind};
use rustc_lint::{LateContext, LateLintPass, LintContext, LintStore};
use rustc_lint_defs::Level;
use rustc_span::{BytePos, Span, Symbol, def_id::LocalDefId, kw};
use source::{Collect, Source};

declare_lint! {
	pub NO_DUPLICATE_STRINGS,
	Warn,
	"repeated string literals should share a named constant"
}

pub(super) fn register(store: &mut LintStore, config: Config) {
	let source: Arc<Mutex<Source>> = Arc::default();
	let collector: Arc<Mutex<Source>> = Arc::clone(&source);
	store.register_pre_expansion_lint_pass(Box::new(move || Box::new(Collect(Arc::clone(&collector)))));
	store.register_late_lint_pass(Box::new(move |_| {
		Box::new(Pass {
			config: config.clone(),
			source: mem::take(&mut *source.lock().unwrap()),
			groups: BTreeMap::new(),
		})
	}));
}

struct Occurrence {
	span: Span,
	id: HirId,
	constant: Option<Symbol>,
}

type Occurrences = BTreeMap<BytePos, Occurrence>;

struct Pass {
	config: Config,
	source: Source,
	groups: BTreeMap<(BytePos, Symbol), Occurrences>,
}
impl_lint_pass!(Pass => [NO_DUPLICATE_STRINGS]);

fn constant_name(cx: &LateContext<'_>, id: HirId) -> Option<Symbol> {
	let owner: LocalDefId = id.owner.def_id;
	if !matches!(cx.tcx.def_kind(owner), DefKind::Const | DefKind::AssocConst) {
		return None;
	}
	let body: &Body<'_> = cx.tcx.hir_maybe_body_owned_by(owner)?;
	if body.value.hir_id != id {
		return None;
	}
	cx.tcx.opt_item_name(owner).filter(|&name| name != kw::Underscore)
}

impl Pass {
	fn record(&mut self, cx: &LateContext<'_>, id: HirId, span: Span, text: Symbol, constant: Option<Symbol>) {
		let ignored: bool = self.config.ignore_strings.contains(text.as_str())
			|| text.as_str().chars().count() < self.config.short_string;
		if ignored {
			return;
		}
		let file: BytePos = cx.sess().source_map().lookup_source_file(span.lo()).start_pos;
		self.groups
			.entry((file, text))
			.or_default()
			.insert(span.lo(), Occurrence { id, span, constant });
	}
}

impl LateLintPass<'_> for Pass {
	fn check_lit(&mut self, cx: &LateContext<'_>, id: HirId, lit: Lit, _: bool) {
		let LitKind::Str(text, _) = lit.node else { return };
		let source_literal: bool = !lit.span.from_expansion() && self.source.literals.contains(&lit.span.lo());
		if !source_literal {
			return;
		}
		if cx.get_lint_level_spec(NO_DUPLICATE_STRINGS).level() == Level::Allow {
			return;
		}
		self.record(cx, id, lit.span, text, constant_name(cx, id));
	}

	fn check_expr(&mut self, cx: &LateContext<'_>, expr: &Expr<'_>) {
		if !expr.span.from_expansion() {
			return;
		}
		if cx.get_lint_level_spec(NO_DUPLICATE_STRINGS).level() == Level::Allow {
			return;
		}
		for (span, text) in self.source.macro_literals(cx, expr.span) {
			self.record(cx, expr.hir_id, span, text, None);
		}
	}

	fn check_crate_post(&mut self, cx: &LateContext<'_>) {
		for occurrences in self.groups.values() {
			let count: usize = occurrences.len();
			if count < self.config.threshold {
				continue;
			}
			report(cx, occurrences, count);
		}
	}
}

fn report(cx: &LateContext<'_>, occurrences: &Occurrences, count: usize) {
	let mut uses = occurrences.values().filter(|site| site.constant.is_none());
	let Some(first) = uses.next() else { return };
	cx.tcx.emit_node_span_lint(
		NO_DUPLICATE_STRINGS,
		first.id,
		first.span,
		DiagDecorator(|diag: &mut Diag<'_>| {
			diag.primary_message(format!("string literal appears {count} times in this file"));
			for site in uses {
				diag.span_label(site.span, "also used here");
			}
			if let Some((span, name)) = occurrences.values().find_map(|site| Some((site.span, site.constant?))) {
				diag.span_label(span, format!("already named by `{name}`"));
				diag.help(format!("reuse `{name}` where it is in scope"));
			} else {
				diag.help("extract a descriptive `const` and reuse it");
			}
		}),
	);
}
