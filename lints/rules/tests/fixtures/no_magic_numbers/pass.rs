#![deny(no_magic_numbers)]
#![allow(dead_code, unused_assignments, unused_variables, explicit_binding_types)]

struct Config { timeout: i32 }
const LENGTH: usize = 32;
static LIMIT: i32 = -42;
enum Status { Missing = 404 }
trait Defaults { const TIMEOUT: i32 = 30; }
impl Defaults for Config { const TIMEOUT: i32 = 60; }

fn main() {
    let mut timeout = -5000;
    timeout = 250;
    let mut config = Config { timeout: 100 };
    config.timeout = -20;
    let _ = [0u8; LENGTH];
    let _ = (-1, 0x00 & LIMIT, 1_u64);
    let _ = (0.0, -1.0, 0.10e1, 10e-1f32, 0e90, 0x01);
    macro_rules! default_delay { () => { i32::from(42_i16) }; }
    let _ = default_delay!();
    #[allow(no_magic_numbers)]
    let _ = 123 + 456;
}
