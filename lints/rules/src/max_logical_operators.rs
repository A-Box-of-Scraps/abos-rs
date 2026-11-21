use std::collections::HashSet;

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{Expr, ExprKind, NodeId, Stmt, StmtKind, UnOp};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::Span;
use serde::Deserialize;

use crate::logical_operators::logical_operands;

declare_lint! {
	pub MAX_LOGICAL_OPERATORS,
	Warn,
	"boolean chain exceeds the configured logical operator limit"
}

#[derive(Clone, Copy, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct Config {
	max: usize,
}

impl Default for Config {
	fn default() -> Self {
		Self { max: 3 }
	}
}

pub(super) struct Pass {
	config: Config,
	counted_children: HashSet<NodeId>,
	reported: HashSet<Span>,
}

impl_lint_pass!(Pass => [MAX_LOGICAL_OPERATORS]);

fn transparent_operand(expr: &Expr) -> Option<&Expr> {
	match &expr.kind {
		ExprKind::Paren(inner) | ExprKind::Unary(UnOp::Not, inner) => Some(inner),
		ExprKind::Block(block, None) => match block.stmts.as_slice() {
			[
				Stmt {
					kind: StmtKind::Expr(inner),
					..
				},
			] => Some(inner),
			_ => None,
		},
		_ => None,
	}
}

impl Pass {
	pub(super) fn new(config: Config) -> Self {
		Self {
			config,
			counted_children: HashSet::new(),
			reported: HashSet::new(),
		}
	}
}

impl EarlyLintPass for Pass {
	fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
		if self.counted_children.remove(&expr.id) {
			return;
		}
		let Some((left, right)) = logical_operands(expr) else {
			return;
		};
		let mut pending: Vec<&Expr> = vec![left, right];
		let mut count: usize = 1;
		while let Some(child) = pending.pop() {
			if let Some((left, right)) = logical_operands(child) {
				self.counted_children.insert(child.id);
				count += 1;
				pending.extend([left, right]);
			} else if let Some(inner) = transparent_operand(child) {
				pending.push(inner);
			}
		}
		let max: usize = self.config.max;
		// A macro may expand the same caller-written expression more than once.
		let new_violation: bool = count > max && self.reported.insert(expr.span);
		if new_violation {
			cx.emit_span_lint(
				MAX_LOGICAL_OPERATORS,
				expr.span,
				DiagDecorator(|diag: &mut Diag<'_>| {
					diag.primary_message(format!("boolean chain has {count} logical operators; maximum is {max}"));
					diag.help("extract a meaningful predicate while preserving short-circuit evaluation");
				}),
			);
		}
	}
}
