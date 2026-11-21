use std::collections::HashSet;

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{
	AssocItem, AssocItemKind, Expr, ExprKind, GenericParam, GenericParamKind, Item, ItemKind, LitKind, Local, NodeId,
	PatKind, UnOp, Variant, token,
};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::{Ident, Span};

declare_lint! {
	pub NO_MAGIC_NUMBERS,
	Warn,
	"a numeric literal other than zero or one in magnitude has no direct name"
}

#[derive(Default)]
pub(crate) struct Pass {
	named: HashSet<NodeId>,
}
impl_lint_pass!(Pass => [NO_MAGIC_NUMBERS]);

impl Pass {
	fn name(&mut self, expr: &Expr) {
		let mut value = expr.peel_parens();
		if let ExprKind::Unary(UnOp::Neg, operand) = &value.kind {
			value = operand.peel_parens();
		}
		if matches!(value.kind, ExprKind::Lit(_)) {
			self.named.insert(value.id);
		}
	}

	fn associated_const(&mut self, item: &AssocItem) {
		if let AssocItemKind::Const(item) = &item.kind
			&& let Some(body) = &item.body
		{
			self.name(body);
		}
	}
}

fn named_field(field: Ident) -> bool {
	!field.name.as_str().starts_with(|c: char| c.is_ascii_digit())
}

// Compare the written decimal value exactly: conversion to f64 could round a nearby literal to one.
fn exempt_float(decimal: &str) -> bool {
	let (mantissa, exponent) = decimal.split_once(['e', 'E']).unwrap_or((decimal, "0"));
	let integer_digits = mantissa.find('.').unwrap_or(mantissa.len());
	let mut digits = mantissa.bytes().filter(|&b| b != b'.').enumerate();
	let Some((position, first)) = digits.find(|&(_, b)| b != b'0') else {
		return true;
	};
	let required_exponent = position as i64 + 1 - integer_digits as i64;
	first == b'1' && digits.all(|(_, b)| b == b'0') && exponent.parse::<i64>().ok() == Some(required_exponent)
}

fn check_literal(cx: &EarlyContext<'_>, span: Span, token: token::Lit) {
	let warn = match LitKind::from_token_lit(token) {
		Ok(LitKind::Int(value, _)) => value.get() > 1,
		Ok(LitKind::Float(value, _)) => !exempt_float(value.as_str()),
		_ => false,
	};
	if warn {
		cx.emit_span_lint(
			NO_MAGIC_NUMBERS,
			span,
			DiagDecorator(|diag: &mut Diag<'_>| {
				diag.primary_message("numeric literal has no direct name");
				diag.help("use a binding or constant that names what this value represents");
			}),
		);
	}
}

impl EarlyLintPass for Pass {
	fn check_local(&mut self, _: &EarlyContext<'_>, local: &Local) {
		if matches!(local.pat.kind, PatKind::Ident(_, _, None))
			&& let Some(value) = local.kind.init()
		{
			self.name(value);
		}
	}

	fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
		let value = match &item.kind {
			ItemKind::Const(item) => &item.body,
			ItemKind::Static(item) => &item.expr,
			_ => return,
		};
		if let Some(value) = value {
			self.name(value);
		}
	}

	fn check_trait_item(&mut self, _: &EarlyContext<'_>, item: &AssocItem) {
		self.associated_const(item);
	}

	fn check_impl_item(&mut self, _: &EarlyContext<'_>, item: &AssocItem) {
		self.associated_const(item);
	}

	fn check_variant(&mut self, _: &EarlyContext<'_>, variant: &Variant) {
		if let Some(value) = &variant.disr_expr {
			self.name(&value.value);
		}
	}

	fn check_generic_param(&mut self, _: &EarlyContext<'_>, param: &GenericParam) {
		if let GenericParamKind::Const {
			default: Some(value), ..
		} = &param.kind
		{
			self.name(&value.value);
		}
	}

	fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
		match &expr.kind {
			ExprKind::Assign(place, value, _) => match &place.peel_parens().kind {
				ExprKind::Path(..) => self.name(value),
				ExprKind::Field(_, field) if named_field(*field) => self.name(value),
				_ => {}
			},
			ExprKind::Struct(record) => {
				for field in record.fields.iter().filter(|field| named_field(field.ident)) {
					self.name(&field.expr);
				}
			}
			ExprKind::Lit(token) if !self.named.remove(&expr.id) && !expr.span.from_expansion() => {
				check_literal(cx, expr.span, *token);
			}
			_ => {}
		}
	}
}
