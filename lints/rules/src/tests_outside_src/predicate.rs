// Unknown target and feature predicates must stay unknown: this lint describes
// the source, not just the target that happens to be compiling it.
use rustc_ast::{Attribute, LitKind, MetaItem, MetaItemInner};
use rustc_span::sym;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Truth {
	Yes,
	No,
	Unknown,
}

impl Truth {
	fn not(self) -> Self {
		match self {
			Self::Yes => Self::No,
			Self::No => Self::Yes,
			Self::Unknown => Self::Unknown,
		}
	}

	fn and(self, other: Self) -> Self {
		match (self, other) {
			(Self::No, _) | (_, Self::No) => Self::No,
			(Self::Yes, value) | (value, Self::Yes) => value,
			_ => Self::Unknown,
		}
	}

	fn or(self, other: Self) -> Self {
		self.not().and(other.not()).not()
	}
}

pub(super) fn evaluate(node: &MetaItemInner, test: Truth) -> Truth {
	if let Some(LitKind::Bool(value)) = node.lit().map(|literal| &literal.kind) {
		return if *value { Truth::Yes } else { Truth::No };
	}
	let Some(meta) = node.meta_item() else {
		return Truth::Unknown;
	};
	let test_flag: bool = meta.is_word() && meta.has_name(sym::test);
	if test_flag {
		return test;
	}
	let Some(children) = meta.meta_item_list() else {
		return Truth::Unknown;
	};
	let mut values = children.iter().map(|child| evaluate(child, test));
	match meta.name() {
		Some(sym::all) => values.fold(Truth::Yes, Truth::and),
		Some(sym::any) => values.fold(Truth::No, Truth::or),
		Some(sym::not) if children.len() == 1 => values.next().unwrap().not(),
		_ => Truth::Unknown,
	}
}

fn permits(attr: &MetaItem, test: Truth) -> Truth {
	match (attr.name(), attr.meta_item_list()) {
		(Some(sym::cfg), Some([guard])) => evaluate(guard, test),
		(Some(sym::cfg_attr), Some([guard, nested @ ..])) => {
			let enabled: Truth = nested
				.iter()
				.filter_map(MetaItemInner::meta_item)
				.map(|attr| permits(attr, test))
				.fold(Truth::Yes, Truth::and);
			evaluate(guard, test).not().or(enabled)
		}
		_ => Truth::Yes,
	}
}

pub(super) fn test_only(attrs: &[Attribute]) -> bool {
	let guards = attrs
		.iter()
		.filter(|a| a.has_name(sym::cfg) || a.has_name(sym::cfg_attr))
		.filter_map(Attribute::meta);
	let allowed = |test| guards.clone().map(|a| permits(&a, test)).fold(Truth::Yes, Truth::and);
	allowed(Truth::No) == Truth::No && allowed(Truth::Yes) != Truth::No
}

pub(super) fn may_redirect(attr: &Attribute) -> bool {
	fn mentions_path(meta: &MetaItem) -> bool {
		meta.has_name(sym::path)
			|| (meta.has_name(sym::cfg_attr)
				&& meta.meta_item_list().is_some_and(|items| {
					items
						.iter()
						.skip(1)
						.filter_map(MetaItemInner::meta_item)
						.any(mentions_path)
				}))
	}
	attr.has_name(sym::path) || (attr.has_name(sym::cfg_attr) && attr.meta().is_some_and(|m| mentions_path(&m)))
}
