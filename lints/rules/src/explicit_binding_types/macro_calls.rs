use std::{
	collections::BTreeMap,
	sync::{Arc, Mutex},
};

use dylint_linting::impl_lint_pass;
use rustc_ast::{Expr, ExprKind};
use rustc_lint::{EarlyContext, EarlyLintPass, LintStore};
use rustc_span::{BytePos, Span};

use super::{EXPLICIT_BINDING_TYPES, Pass};

// A macro can return an input token with its original span, so expansion
// hygiene alone cannot distinguish `identity!(1)` from a literal initializer.
#[derive(Clone, Default)]
pub(super) struct MacroCalls(BTreeMap<BytePos, BytePos>);

impl MacroCalls {
	pub(super) fn hides(&self, expression: Span, binding: Span) -> bool {
		self.0
			.range(..=expression.lo())
			.next_back()
			.is_some_and(|(&start, &end)| binding.lo() < start && expression.hi() <= end)
	}
}

struct Collect(Arc<Mutex<MacroCalls>>);
impl_lint_pass!(Collect => [EXPLICIT_BINDING_TYPES]);

impl EarlyLintPass for Collect {
	fn check_expr(&mut self, _: &EarlyContext<'_>, expr: &Expr) {
		if let ExprKind::MacCall(call) = &expr.kind {
			let span: Span = call.span();
			self.0.lock().unwrap().0.insert(span.lo(), span.hi());
		}
	}
}

pub(crate) fn register(store: &mut LintStore) {
	let shared: Arc<Mutex<MacroCalls>> = Arc::default();
	let collector: Arc<Mutex<MacroCalls>> = Arc::clone(&shared);
	store.register_pre_expansion_lint_pass(Box::new(move || Box::new(Collect(Arc::clone(&collector)))));
	store.register_late_lint_pass(Box::new(move |_| Box::new(Pass(shared.lock().unwrap().clone()))));
}
