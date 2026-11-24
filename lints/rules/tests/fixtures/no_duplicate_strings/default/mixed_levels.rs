#[warn(no_duplicate_strings)]
fn warn_first() {
	let _ = "warning first";
}

#[deny(no_duplicate_strings)]
fn denied() {
	let _ = ("warning first", "warning first");
	let _ = ("denial first", "denial first");
}

#[warn(no_duplicate_strings)]
fn warn_last() {
	let _ = "denial first";
}

fn main() {
	warn_first();
	denied();
	warn_last();
}
