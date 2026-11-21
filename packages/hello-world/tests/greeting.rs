use std::process::Command;

#[test]
fn prints_greeting() {
	let output = Command::new(env!("CARGO_BIN_EXE_hello-world"))
		.output()
		.expect("failed to run hello-world");

	assert!(output.status.success(), "{output:?}");
	assert_eq!(output.stdout, b"Hello, world!\n");
	assert!(output.stderr.is_empty(), "{output:?}");
}
