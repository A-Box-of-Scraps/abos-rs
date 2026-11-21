mod predicate;

use std::{
	collections::HashSet,
	env,
	path::PathBuf,
	sync::{Arc, Mutex, MutexGuard},
};

use dylint_linting::{declare_lint, impl_lint_pass};
use rustc_ast::{
	AssocItem, AttrKind, Attribute, Crate, Expr, ExprKind, HasAttrs, Inline, Item, ItemKind, MetaItemKind, ModKind,
	Stmt, SyntheticAttr,
};
use rustc_errors::{Diag, DiagDecorator};
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext, LintStore};
use rustc_span::{BytePos, FileName, SourceFile, Span, sym};

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
	fn source_path(cx: &EarlyContext<'_>, span: Span) -> Option<PathBuf> {
		if span.is_dummy() {
			return None;
		}
		let file: Arc<SourceFile> = cx.sess().source_map().lookup_source_file(span.lo());
		let FileName::Real(name) = &file.name else { return None };
		name.local_path()?.canonicalize().ok()
	}

	fn report(&self, cx: &EarlyContext<'_>, region: Span, label: Span, message: &'static str) {
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
	store.register_pre_expansion_lint_pass(Box::new(move || {
		Box::new(SourcePass {
			locations: source.clone(),
			redirected_depth: 0,
		})
	}));
	store.register_early_lint_pass(Box::new(move || Box::new(HarnessPass(locations.clone()))));
}

struct SourcePass {
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
		if redirected && matches!(kind, ModKind::Unloaded) {
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
		if self.redirected_depth > 0 && matches!(item.kind, ItemKind::Mod(_, _, ModKind::Loaded(..))) {
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
		if call.path != sym::cfg {
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
			self.locations
				.report(cx, span, span, "move this test-only branch outside src/");
		}
	}
}

struct HarnessPass(Locations);
impl_lint_pass!(HarnessPass => [TESTS_OUTSIDE_SRC]);

impl EarlyLintPass for HarnessPass {
	fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
		if let ItemKind::Mod(_, _, ModKind::Loaded(_, Inline::No { .. }, spans)) = &item.kind
			&& self.0.redirected_modules.lock().unwrap().contains(&item.span.lo())
			&& Locations::source_path(cx, spans.inner_span).is_some_and(|path| path.starts_with(&self.0.src))
		{
			self.0
				.report(cx, spans.inner_span, item.span, "move this test-only code outside src/");
		}
		if let ItemKind::Fn(function) = &item.kind
			&& item.attrs.iter().any(
				|a| matches!(&a.kind, AttrKind::Synthetic(kind) if matches!(**kind, SyntheticAttr::TestEntrypoint)),
			) {
			let label: Span = function.ident.span.source_callsite();
			self.0
				.report(cx, item.span.source_callsite(), label, "move this test outside src/");
		}
	}
}
