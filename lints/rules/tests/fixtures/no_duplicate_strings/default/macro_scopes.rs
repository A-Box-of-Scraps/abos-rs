fn locally_allowed() {
	println!("{}", {
		#[allow(no_duplicate_strings)]
		let value = "allowed locally";
		value
	});
	let _ = ("allowed locally", "allowed locally");
}

#[allow(no_duplicate_strings)]
fn locally_denied() {
	println!("{}", {
		#[deny(no_duplicate_strings)]
		let value = "denied locally";
		value
	});
}

fn main() {
	locally_allowed();
	locally_denied();
	let _ = ("denied locally", "denied locally");
}
