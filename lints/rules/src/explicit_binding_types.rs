mod macro_calls;

use macro_calls::MacroCalls;
pub(super) use macro_calls::register;

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_errors::{Diag, DiagDecorator};
use rustc_hir::{
	self as hir, BindingMode, Expr, ExprKind, GenericArg, LetStmt, LocalSource, PatKind, PathSegment, QPath,
	def::{DefKind, Res},
	intravisit::{InferKind, Visitor},
};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_middle::ty::{self, Ty};
use rustc_span::{Span, sym};

declare_lint! {
	pub EXPLICIT_BINDING_TYPES,
	Warn,
	"a simple binding has neither an explicit type nor an initializer that shows its type"
}

struct Pass(MacroCalls);
impl_lint_pass!(Pass => [EXPLICIT_BINDING_TYPES]);

#[derive(Default)]
struct Placeholders(bool);

impl<'hir> Visitor<'hir> for Placeholders {
	fn visit_infer(&mut self, _: hir::HirId, _: Span, _: InferKind<'hir>) {
		self.0 = true;
	}
}

fn incomplete(ty: &hir::Ty<'_>) -> bool {
	let mut placeholders: Placeholders = Placeholders::default();
	placeholders.visit_ty_unambig(ty);
	placeholders.0
}

fn shows_adt<'tcx>(cx: &LateContext<'tcx>, segment: &PathSegment<'tcx>, result: Ty<'tcx>) -> bool {
	let ty::Adt(adt, args) = *result.kind() else {
		return false;
	};
	match segment.res {
		Res::SelfTyAlias { .. } => return !adt.is_union(),
		Res::Def(DefKind::Struct | DefKind::Enum, id) if id == adt.did() => {}
		_ => return false,
	}
	let mut placeholders: Placeholders = Placeholders::default();
	let written: usize = segment.args.map_or(0, |args| {
		placeholders.visit_generic_args(args);
		args.args
			.iter()
			.filter(|a| !matches!(a, GenericArg::Lifetime(_)))
			.count()
	});
	if placeholders.0 {
		return false;
	}
	let mut params = cx
		.tcx
		.generics_of(adt.did())
		.own_params
		.iter()
		.filter(|p| p.kind.is_ty_or_const());
	if written == 0 {
		return params.next().is_none();
	}
	params.skip(written).all(|param| {
		param.default_value(cx.tcx).is_some_and(|default| {
			cx.tcx
				.try_normalize_erasing_regions(cx.typing_env(), default.instantiate(cx.tcx, args))
				.is_ok_and(|value| value == args[param.index as usize])
		})
	})
}

fn named_type<'tcx>(cx: &LateContext<'tcx>, ty: &hir::Ty<'tcx>, result: Ty<'tcx>) -> bool {
	let hir::TyKind::Path(QPath::Resolved(None, path)) = ty.kind else {
		return false;
	};
	// The same ADT can be returned with different type or const arguments.
	cx.typeck_results().node_type_opt(ty.hir_id) == Some(result)
		&& path.segments.last().is_some_and(|s| shows_adt(cx, s, result))
}

fn struct_literal<'tcx>(cx: &LateContext<'tcx>, path: &QPath<'tcx>, result: Ty<'tcx>) -> bool {
	match path {
		QPath::TypeRelative(ty, _) => named_type(cx, ty, result),
		QPath::Resolved(None, path) => {
			let segment: &PathSegment<'_> = match (path.res, path.segments) {
				(Res::Def(DefKind::Variant, _), [.., base, _]) => base,
				(_, [.., last]) => last,
				_ => return false,
			};
			shows_adt(cx, segment, result)
		}
		_ => false,
	}
}

fn shows_type<'tcx>(cx: &LateContext<'tcx>, expr: &Expr<'tcx>, macros: &MacroCalls, binding: Span) -> bool {
	if expr.span.from_expansion() || macros.hides(expr.span, binding) {
		return false;
	}
	match expr.kind {
		ExprKind::Lit(_) => true,
		ExprKind::Type(_, ty) | ExprKind::Cast(_, ty) => !incomplete(ty),
		ExprKind::AddrOf(_, _, inner)
		| ExprKind::Unary(hir::UnOp::Neg | hir::UnOp::Not, inner)
		| ExprKind::Repeat(inner, _) => shows_type(cx, inner, macros, binding),
		ExprKind::Tup(parts) => parts.iter().all(|part| shows_type(cx, part, macros, binding)),
		ExprKind::Array(parts) => !parts.is_empty() && parts.iter().all(|part| shows_type(cx, part, macros, binding)),
		ExprKind::Struct(path, ..) => struct_literal(cx, path, cx.typeck_results().expr_ty(expr)),
		ExprKind::Call(callee, _) if !callee.span.from_expansion() && !macros.hides(callee.span, binding) => {
			matches!(callee.kind, ExprKind::Path(QPath::TypeRelative(ty, segment))
				if segment.ident.name == sym::new && named_type(cx, ty, cx.typeck_results().expr_ty(expr)))
		}
		_ => false,
	}
}

fn contains_unnameable(ty: Ty<'_>) -> bool {
	ty.walk().filter_map(ty::GenericArg::as_type).any(|ty| {
		ty.is_opaque()
			|| matches!(
				ty.kind(),
				ty::Closure(..) | ty::CoroutineClosure(..) | ty::Coroutine(..)
			)
	})
}

impl<'tcx> LateLintPass<'tcx> for Pass {
	fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx LetStmt<'tcx>) {
		let PatKind::Binding(BindingMode::NONE | BindingMode::MUT, _, name, None) = local.pat.kind else {
			return;
		};
		let Some(init) = local.init else { return };
		if !matches!(local.source, LocalSource::Normal)
			|| local.span.from_expansion()
			|| name.span.from_expansion()
			|| local.ty.is_some_and(|ty| !incomplete(ty))
			|| contains_unnameable(cx.typeck_results().pat_ty(local.pat))
		{
			return;
		}
		let shown: bool = shows_type(cx, init, &self.0, local.span);
		if local.ty.is_none() && shown {
			return;
		}
		let (span, message, help) = match local.ty {
			Some(ty) => (
				ty.span,
				"binding type contains a placeholder",
				if shown {
					"write the complete type or remove the annotation; the initializer already shows the type"
				} else {
					"replace each placeholder with an explicit type or const argument"
				},
			),
			None => (
				name.span,
				"initializer does not show the binding's type",
				"add a type annotation to this binding",
			),
		};
		cx.emit_span_lint(
			EXPLICIT_BINDING_TYPES,
			span,
			DiagDecorator(|diag: &mut Diag<'_>| {
				diag.primary_message(message);
				diag.help(help);
			}),
		);
	}
}
