use std::{collections::HashSet, ops::ControlFlow};

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{
	Arm, Block, Expr, ExprKind, StmtKind,
	visit::{self, Visitor},
};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;

use crate::logical_operators::logical_operands;

declare_lint! {
	pub NO_INLINE_LOGICAL_CONDITIONS,
	Warn,
	"inline logical operators obscure the intent of a control-flow condition"
}

#[derive(Default)]
pub(super) struct Pass {
	reported: HashSet<Span>,
}
impl_lint_pass!(Pass => [NO_INLINE_LOGICAL_CONDITIONS]);

struct InlineOperator;

impl<'ast> Visitor<'ast> for InlineOperator {
	type Result = ControlFlow<Span>;

	fn visit_expr(&mut self, expr: &'ast Expr) -> Self::Result {
		if logical_operands(expr).is_some() {
			return ControlFlow::Break(expr.span);
		}
		match &expr.kind {
			// Nested decisions own their conditions; their branch values can still
			// contribute directly to the enclosing condition.
			ExprKind::If(_, then, otherwise) => {
				self.visit_block(then)?;
				otherwise
					.as_deref()
					.map_or(ControlFlow::Continue(()), |value| self.visit_expr(value))
			}
			ExprKind::Closure(..)
			| ExprKind::Gen(..)
			| ExprKind::ConstBlock(..)
			| ExprKind::While(..)
			| ExprKind::ForLoop { .. }
			| ExprKind::Loop(..) => ControlFlow::Continue(()),
			_ => visit::walk_expr(self, expr),
		}
	}

	fn visit_block(&mut self, block: &'ast Block) -> Self::Result {
		// Only the tail value is the decision, not declarations used to compute it.
		match block.stmts.last().map(|stmt| &stmt.kind) {
			Some(StmtKind::Expr(value)) => self.visit_expr(value),
			_ => ControlFlow::Continue(()),
		}
	}

	fn visit_arm(&mut self, arm: &'ast Arm) -> Self::Result {
		if let Some(body) = &arm.body {
			self.visit_expr(body)?;
		}
		ControlFlow::Continue(())
	}
}

impl Pass {
	fn check_condition(&mut self, cx: &EarlyContext<'_>, condition: &Expr, help: &'static str) {
		let ControlFlow::Break(span) = InlineOperator.visit_expr(condition) else {
			return;
		};
		if !self.reported.insert(span) {
			return;
		}
		cx.emit_span_lint(
			NO_INLINE_LOGICAL_CONDITIONS,
			span,
			DiagDecorator(|diag: &mut Diag<'_>| {
				diag.primary_message("avoid inline logical operators in this condition");
				diag.help(help);
			}),
		);
	}
}

const NAME_DECISION: &str =
	"name the decision with a boolean binding or predicate; if a pattern introduces a binding, consider a nested check";
const LOOP_DECISION: &str = "use a named predicate that is checked each iteration; for a pattern binding, consider a nested check that preserves when the loop exits";

impl EarlyLintPass for Pass {
	fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
		if expr.span.from_expansion() {
			return;
		}
		match &expr.kind {
			ExprKind::If(condition, ..) => self.check_condition(cx, condition, NAME_DECISION),
			ExprKind::While(condition, ..) => self.check_condition(cx, condition, LOOP_DECISION),
			_ => {}
		}
	}

	fn check_arm(&mut self, cx: &EarlyContext<'_>, arm: &Arm) {
		if let Some(guard) = &arm.guard {
			self.check_condition(cx, &guard.cond, NAME_DECISION);
		}
	}
}
