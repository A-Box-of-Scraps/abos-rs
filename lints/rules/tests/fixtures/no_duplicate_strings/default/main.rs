#[path = "auxiliary/other.rs"]
mod other;
const REQUEST_ID: &str = "request-id";
const A: &str = "constant only";
const B: &str = "constant only";
const C: &str = "constant only";

macro_rules! literal { ($s:literal) => { $s }; }
macro_rules! generated { () => { "request-id" }; }

fn send(_: &str) {}
fn main() {
	send(r"request-id");
	assert_eq!(REQUEST_ID, "request\x2did");
	let _ = (literal!("request-id"), generated!());
	println!("request-id");
	let _ = ("ééééééééé", "ééééééééé", "ééééééééé");
	let _ = ("across files", "across files");
	let _ = (b"byte string", b"byte string", b"byte string");
	send("across tests");
	println!("{}", "formatted value");
	let _ = ("formatted value", "formatted value");
}

#[cfg(test)]
mod tests {
	const USED: &str = "across tests";
	fn test() { super::send("across tests"); }
}

#[allow(no_duplicate_strings)]
fn excluded() { send("request-id"); }

mod shadowed {
	macro_rules! assert_eq { ($a:literal, $b:literal) => { ($a, $b) }; }
	const LABELS: (&str, &str) = assert_eq!("request-id", "request-id");
}
