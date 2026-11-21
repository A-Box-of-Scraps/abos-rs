// compile-flags: --edition=2024
#![deny(explicit_binding_types)]
use std::sync::MutexGuard;

struct Slot<T = u8, const N: usize = 2> { value: T }
impl<T, const N: usize> Slot<T, N> {
    fn new(value: T) -> Self { Self { value } }
    fn wrap(value: T) {
        let slot = Self::new(value);
        let again = Self { value: slot.value };
    }
}
enum Status<T> { Ready, Pending { value: T } }
type Bytes = Vec<u8>;

fn opaque() -> Option<impl Iterator<Item = u8>> { Some([1].into_iter()) }
fn number() -> u8 { 5 }
fn run(guard: MutexGuard<'_, u8>) {
    let other: MutexGuard<'_, u8> = guard;
    let mut num = -1;
    let pair = (&true, ["toto"; 2]);
    let items = [1, 2];
    let flipped = !true;
    let cast = number() as i32;
    let players = Vec::<u8>::new();
    let slot = Slot::<u8> { value: 1 };
    let slot = Slot::<u8>::new(1);
    let slot = Slot::<u16, 4> { value: 1 };
    let state = Status::<u8>::Pending { value: 4 };
    let bytes: Bytes = Vec::new();
    let slot: Slot = Slot::new(1);
    let (x, _) = pair;
    let _ = number();
    let later;
    later = number();
    let task = async { 1 };
    let callbacks = [|| true];
    let it = opaque();
    if let Some(value) = it { for x in value {} }
    let mut it = opaque();
    while let Some(value) = it.take() {}
    match number() { v => () }
    macro_rules! bind { () => { let generated = number(); } }
    bind!();
    #[allow(explicit_binding_types)]
    let allowed = number();
}
fn main() {}
