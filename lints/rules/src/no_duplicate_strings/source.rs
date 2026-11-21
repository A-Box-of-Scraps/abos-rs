use std::{
	collections::{HashMap, HashSet},
	sync::{Arc, Mutex},
};

use dylint_linting::impl_lint_pass;
use rustc_ast::{
	self as ast, ExprKind, LitKind, MacCall, token,
	tokenstream::TokenStream,
	visit::{self, Visitor},
};
use rustc_lint::{EarlyContext, EarlyLintPass, LateContext, LintContext};
use rustc_parse::parser::Parser;
use rustc_span::{BytePos, Span, Symbol, sym};

use super::NO_DUPLICATE_STRINGS;

// A macro can forward a literal with its original span even when its matcher
// requires a literal. Only source expressions and verified macro slots qualify.
#[derive(Default)]
pub(super) struct Source {
	pub literals: HashSet<BytePos>,
	pub calls: HashMap<BytePos, TokenStream>,
}

pub(super) struct Collect(pub Arc<Mutex<Source>>);
impl_lint_pass!(Collect => [NO_DUPLICATE_STRINGS]);

impl EarlyLintPass for Collect {
	fn check_expr(&mut self, _: &EarlyContext<'_>, expr: &ast::Expr) {
		if let ExprKind::Lit(token::Lit {
			kind: token::Str | token::StrRaw(_),
			..
		}) = expr.kind
		{
			self.0.lock().unwrap().literals.insert(expr.span.lo());
		}
	}

	fn check_mac(&mut self, _: &EarlyContext<'_>, call: &MacCall) {
		self.0
			.lock()
			.unwrap()
			.calls
			.insert(call.span().lo(), call.args.tokens.clone());
	}
}

struct Arguments<'a> {
	literals: &'a mut HashSet<BytePos>,
	direct: Vec<(Span, Symbol)>,
}

impl<'ast> Visitor<'ast> for Arguments<'_> {
	fn visit_expr(&mut self, expr: &'ast ast::Expr) {
		if let ExprKind::Lit(token) = expr.kind {
			if matches!(token.kind, token::Str | token::StrRaw(_)) {
				self.literals.insert(expr.span.lo());
			}
		} else {
			visit::walk_expr(self, expr);
		}
	}

	fn visit_item(&mut self, _: &'ast ast::Item) {}
	fn visit_mac_call(&mut self, _: &'ast MacCall) {}
}

fn template_slot(name: Symbol) -> Option<usize> {
	match name {
		sym::assert_eq_macro | sym::assert_ne_macro | sym::debug_assert_eq_macro | sym::debug_assert_ne_macro => {
			const COMPARISON_OPERANDS: usize = 2;
			Some(COMPARISON_OPERANDS)
		}
		sym::assert_macro | sym::debug_assert_macro | sym::write_macro | sym::writeln_macro => Some(1),
		sym::print_macro
		| sym::println_macro
		| sym::eprint_macro
		| sym::eprintln_macro
		| sym::format_macro
		| sym::format_args_macro => Some(0),
		_ => None,
	}
}

// rustc can fold literals through parentheses and references, including named
// arguments. Only use the macro's lint scope when no wrapper has attributes.
fn foldable_string(expr: &ast::Expr) -> Option<(Span, Symbol)> {
	if !expr.attrs.is_empty() {
		return None;
	}
	let mut value: &ast::Expr = match &expr.kind {
		ExprKind::Assign(_, value, _) => value.as_ref(),
		_ => expr,
	};
	while value.attrs.is_empty() {
		match &value.kind {
			ExprKind::Paren(inner) | ExprKind::AddrOf(ast::BorrowKind::Ref, _, inner) => value = inner,
			ExprKind::Lit(token) => {
				let LitKind::Str(text, _) = LitKind::from_token_lit(*token).ok()? else {
					return None;
				};
				return Some((value.span, text));
			}
			_ => return None,
		}
	}
	None
}

impl Arguments<'_> {
	fn collect_format_value(&mut self, expr: &ast::Expr) {
		if let Some(literal) = foldable_string(expr) {
			self.direct.push(literal);
		} else {
			// Let check_lit use the surviving literal's own HIR scope.
			self.visit_expr(expr);
		}
	}

	fn collect_arguments(&mut self, cx: &LateContext<'_>, tokens: TokenStream, template: usize) {
		// Read original arguments: rustc may fold formatting values into the template.
		let mut parser = Parser::new(&cx.sess().psess, tokens, None);
		let mut index: usize = 0;
		while parser.token.kind != token::Eof {
			let expr: Box<ast::Expr> = match parser.parse_expr() {
				Ok(expr) => expr,
				Err(error) => {
					error.cancel();
					break;
				}
			};
			if index < template {
				self.visit_expr(&expr);
			} else if index > template {
				self.collect_format_value(&expr);
			}
			if parser.token.kind != token::Comma {
				break;
			}
			parser.bump();
			index += 1;
		}
	}
}

impl Source {
	pub(super) fn collect_macro_arguments(&mut self, cx: &LateContext<'_>, span: Span) -> Vec<(Span, Symbol)> {
		let mut found = Arguments {
			literals: &mut self.literals,
			direct: Vec::new(),
		};
		for expansion in span.macro_backtrace() {
			if expansion.call_site.from_expansion() {
				continue;
			}
			let Some(def) = expansion.macro_def_id else { continue };
			let Some(template) = cx.tcx.get_diagnostic_name(def).and_then(template_slot) else {
				continue;
			};
			let Some(tokens) = self.calls.remove(&expansion.call_site.lo()) else {
				continue;
			};
			found.collect_arguments(cx, tokens, template);
		}
		found.direct
	}
}
