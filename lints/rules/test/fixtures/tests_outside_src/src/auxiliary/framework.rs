// no-prefer-dynamic
// compile-flags: --emit=link
#![crate_type = "proc-macro"]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn check(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut registered: TokenStream = "#[test]".parse().unwrap();
    registered.extend(item);
    registered
}
