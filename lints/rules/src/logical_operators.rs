use rustc_ast::{BinOpKind, Expr, ExprKind};

pub(super) fn logical_operands(expr: &Expr) -> Option<(&Expr, &Expr)> {
	let ExprKind::Binary(op, left, right) = &expr.kind else {
		return None;
	};
	let user_written: bool = matches!(op.node, BinOpKind::And | BinOpKind::Or) && !op.span.from_expansion();
	user_written.then_some((left, right))
}
