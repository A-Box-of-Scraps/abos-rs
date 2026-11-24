#![allow(unused_braces)]

macro_rules! identity { ($e:expr) => { $e }; }
macro_rules! generated { ($a:expr, $b:expr) => { $a && $b }; }

fn accepts(value: bool) -> bool { value }
fn check(a: bool, b: bool, retries: usize, limit: usize) {
	let can_ship = a && b;
	if can_ship {}
	if retries < limit {}
	if { let eligible = a || b; eligible } {}
	if [a].into_iter().any(|x| x && b) {}
	for _ in [a].into_iter().filter(|x| *x && b) {}
	if generated!(a, b) {}
	assert!(a && b);

	if a && b || a {} else if !accepts(a || b) {}
	if { a && b } {}
	if identity!(a || b) {}
	if let Some(x) = Some(a) && x {}
	while a && b { break; }
	while let Some(x) = Some(a) && x { break; }
	match a {
		true if accepts(a && b) => (),
		_ => (),
	}
	for x in [a] { if x && b {} }
	loop { if a || b { break; } }
}

#[warn(max_logical_operators)]
fn both_rules(a: bool, b: bool) {
	if a && b || a && b || a {}
}

fn nested_decisions(a: bool, b: bool) {
	// The inner condition owns its warning; the outer tests the branch value.
	if (if a && b { true } else { false }) {}
	if (if a { true } else { a || b }) {}
	if (match a { true if b => a && b, _ => false }) {}
}

fn main() {}
