mod predicate;

use std::{
	collections::{HashMap, HashSet},
	env, mem,
	path::PathBuf,
	sync::{Arc, Mutex, MutexGuard},
};

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{
	AssocItem, AttrKind, Attribute, Crate, Expr, ExprKind, HasAttrs, Inline, Item, ItemKind, MetaItemKind, ModKind,
	Stmt, SyntheticAttr,
};
use rustc_attr_ir::find_attr;
use rustc_errors::{Diag, DiagDecorator};
use rustc_hir as hir;
use rustc_lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass, LintContext, LintStore};
use rustc_span::{BytePos, FileName, SourceFile, Span, hygiene::ExpnData, sym};

use self::predicate::{Truth, evaluate, test_only};

declare_lint! {
	pub TESTS_OUTSIDE_SRC,
	Warn,
	"tests and test-only code inside the package's src directory"
}

// Pre-expansion can see inactive cfgs; only post-expansion can identify tests
// registered by arbitrary procedural macros. Share reported regions across both.
#[derive(Clone)]
struct Locations {
	src: PathBuf,
	reported: Arc<Mutex<Vec<(BytePos, BytePos)>>>,
	redirected_modules: Arc<Mutex<HashSet<BytePos>>>,
}

impl Locations {
	fn source_path(cx: &impl LintContext, span: Span) -> Option<PathBuf> {
		if span.is_dummy() {
			return None;
		}
		let file: Arc<SourceFile> = cx.sess().source_map().lookup_source_file(span.lo());
		let FileName::Real(name) = &file.name else { return None };
		name.local_path()?.canonicalize().ok()
	}

	fn report(&self, cx: &impl LintContext, region: Span, label: Span, message: &'static str) {
		if !Self::source_path(cx, label).is_some_and(|path| path.starts_with(&self.src)) {
			return;
		}
		let mut regions: MutexGuard<'_, Vec<(BytePos, BytePos)>> = self.reported.lock().unwrap();
		if regions.iter().any(|&(lo, hi)| lo <= label.lo() && label.hi() <= hi) {
			return;
		}
		regions.push((region.lo(), region.hi()));
		drop(regions);
		cx.emit_span_lint(
			TESTS_OUTSIDE_SRC,
			label,
			DiagDecorator(|diag: &mut Diag<'_>| {
				diag.primary_message(message);
			}),
		);
	}
}

pub(super) fn register(store: &mut LintStore) {
	let Some(src) =
		env::var_os("CARGO_MANIFEST_DIR").and_then(|dir| PathBuf::from(dir).join("src").canonicalize().ok())
	else {
		return;
	};
	let locations = Locations {
		src,
		reported: Arc::default(),
		redirected_modules: Arc::default(),
	};
	let source: Locations = locations.clone();
	let branches: Arc<Mutex<HashMap<Span, Span>>> = Arc::default();
	let candidates: Arc<Mutex<HashMap<Span, Span>>> = Arc::clone(&branches);
	store.register_pre_expansion_lint_pass(Box::new(move || {
		Box::new(SourcePass {
			locations: source.clone(),
			branches: Arc::clone(&candidates),
			redirected_depth: 0,
		})
	}));
	let harness: Locations = locations.clone();
	store.register_early_lint_pass(Box::new(move || Box::new(HarnessPass(harness.clone()))));
	store.register_late_lint_pass(Box::new(move |_| {
		Box::new(BranchPass {
			locations: locations.clone(),
			branches: mem::take(&mut *branches.lock().unwrap()),
		})
	}));
}

struct SourcePass {
	branches: Arc<Mutex<HashMap<Span, Span>>>,
	locations: Locations,
	redirected_depth: usize,
}
impl_lint_pass!(SourcePass => [TESTS_OUTSIDE_SRC]);

impl SourcePass {
	fn guarded(&self, cx: &EarlyContext<'_>, span: Span, attrs: &[Attribute]) {
		if test_only(attrs) {
			self.locations
				.report(cx, span, span, "move this test-only code outside src/");
		}
	}
}

impl EarlyLintPass for SourcePass {
	fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &Crate) {
		self.guarded(cx, krate.spans.inner_span, &krate.attrs);
	}

	fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
		let ItemKind::Mod(_, _, kind) = &item.kind else {
			self.guarded(cx, item.span, &item.attrs);
			return;
		};
		// Relative paths inside inline modules depend on how the enclosing
		// module was loaded. For redirected modules, use rustc's resolved file.
		let redirected: bool = self.redirected_depth > 0 || item.attrs.iter().any(predicate::may_redirect);
		let defer_to_resolved_path: bool = redirected && matches!(kind, ModKind::Unloaded);
		if defer_to_resolved_path {
			if test_only(&item.attrs) {
				self.locations.redirected_modules.lock().unwrap().insert(item.span.lo());
			}
			return;
		}
		if redirected {
			self.redirected_depth += 1;
		}
		self.guarded(cx, item.span, &item.attrs);
	}

	fn check_item_post(&mut self, _: &EarlyContext<'_>, item: &Item) {
		let leaving_redirected_module: bool =
			self.redirected_depth > 0 && matches!(item.kind, ItemKind::Mod(_, _, ModKind::Loaded(..)));
		if leaving_redirected_module {
			self.redirected_depth -= 1;
		}
	}

	fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &AssocItem) {
		self.guarded(cx, item.span, &item.attrs);
	}
	fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &AssocItem) {
		self.guarded(cx, item.span, &item.attrs);
	}
	fn check_stmt(&mut self, cx: &EarlyContext<'_>, stmt: &Stmt) {
		if !stmt.is_item() {
			self.guarded(cx, stmt.span, stmt.attrs());
		}
	}
	fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
		self.guarded(cx, expr.span, &expr.attrs);
		let ExprKind::If(condition, then, otherwise) = &expr.kind else {
			return;
		};
		let ExprKind::MacCall(call) = &condition.kind else {
			return;
		};
		if !call
			.path
			.segments
			.last()
			.is_some_and(|segment| segment.ident.name == sym::cfg)
		{
			return;
		}
		let Some(args) = MetaItemKind::list_from_tokens(call.args.tokens.clone()) else {
			return;
		};
		let [guard] = &args[..] else { return };
		let branch: Option<Span> = match (evaluate(guard, Truth::No), evaluate(guard, Truth::Yes)) {
			(Truth::No, Truth::Yes | Truth::Unknown) => Some(then.span),
			(Truth::Yes, Truth::No | Truth::Unknown) => otherwise.as_ref().map(|e| e.span),
			_ => None,
		};
		if let Some(span) = branch {
			self.branches.lock().unwrap().insert(call.span(), span);
		}
	}
}

struct HarnessPass(Locations);
impl_lint_pass!(HarnessPass => [TESTS_OUTSIDE_SRC]);

impl EarlyLintPass for HarnessPass {
	fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
		if let ItemKind::Mod(_, _, ModKind::Loaded(_, Inline::No { .. }, spans)) = &item.kind {
			let redirected_into_src: bool = self.0.redirected_modules.lock().unwrap().contains(&item.span.lo())
				&& Locations::source_path(cx, spans.inner_span).is_some_and(|path| path.starts_with(&self.0.src));
			if redirected_into_src {
				self.0
					.report(cx, spans.inner_span, item.span, "move this test-only code outside src/");
			}
		}
		let ItemKind::Fn(function) = &item.kind else { return };
		if item
			.attrs
			.iter()
			.any(|a| matches!(&a.kind, AttrKind::Synthetic(kind) if matches!(**kind, SyntheticAttr::TestEntrypoint)))
		{
			let label: Span = function.ident.span.source_callsite();
			self.0
				.report(cx, item.span.source_callsite(), label, "move this test outside src/");
		}
	}
}

// The source pass can read the predicate, but cannot distinguish built-in cfg!
// from a user macro with the same name. Match the direct expansion at its call site.
struct BranchPass {
	locations: Locations,
	branches: HashMap<Span, Span>,
}
impl_lint_pass!(BranchPass => [TESTS_OUTSIDE_SRC]);

impl LateLintPass<'_> for BranchPass {
	fn check_expr(&mut self, cx: &LateContext<'_>, expr: &hir::Expr<'_>) {
		let hir::ExprKind::If(condition, ..) = expr.kind else {
			return;
		};
		let expansion: ExpnData = condition.span.ctxt().outer_expn_data();
		let Some(branch) = self.branches.remove(&expansion.call_site) else {
			return;
		};
		let Some(id) = expansion.macro_def_id else { return };
		let builtin_cfg: bool = cx.tcx.item_name(id) == sym::cfg && find_attr!(cx.tcx, id, RustcBuiltinMacro { .. });
		if builtin_cfg {
			self.locations
				.report(cx, branch, branch, "move this test-only branch outside src/");
		}
	}
}
