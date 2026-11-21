#![allow(unused_braces)]

fn accepts(value: bool) -> bool { value }

macro_rules! generated {
	($a:expr) => { $a && $a && $a && $a && $a };
}

fn main() {
	let (a, b, c, d, e) = (true, false, true, false, true);
	let _ = a && b || c && d;
	let _ = a && accepts(b || c || d || e);
	let _ = a && { let eligible = b || c || d || e; eligible };
	let _ = generated!(a);

	let _ = a && b && c && d && e;
	let _ = a || b || c || d || e;
	let _ = a && !(b || c || d || e);
	let _ = a && { b || c || d || e };
	if let Some(value) = Some(a) && value && c && d && e {}
	assert!(a && b || c && d || e);
	let _ = generated!(a && b || c && d || e);
}
