// compile-flags: --edition=2024
struct Player { score: u8 }
impl Player {
    fn new() -> Self { Self { score: 1 } }
    fn get(&self) -> Self { Self::new() }
}
type Alias = Player;
struct Batch<T, const N: usize> { items: [T; N] }
struct Changing<T> { value: T }
impl Changing<u8> {
    fn new() -> Changing<u16> { Changing { value: 1 } }
}
enum State { Ready, Failed(u16) }

fn get() -> Player { Player::new() }
async fn fetch() -> Option<Player> { Some(get()) }
async fn run() -> Option<()> {
    let mut a = get();
    let b = a.get();
    let field = b.score;
    let copy = b;
    let inferred = Vec::new();
    inferred.into_iter().for_each(|_: Player| ());
    let partial = Vec::<_>::new();
    partial.into_iter().for_each(|_: Player| ());
    let nested: Vec<Option<_>> = Vec::<Option<Player>>::new();
    let generic = Batch { items: [0] };
    let generic = Batch::<u8, _> { items: [0] };
    let annotated: Batch<u8, _> = generic;
    let wrong_type = Changing::<u8>::new();
    let alias = Alias::new();
    let alias = Alias { score: 1 };
    let ready = State::Ready;
    let failed = State::Failed(1);
    let control = if true { get() } else { get() };
    let awaited = fetch().await?;
    let mixed = (1, [get()]);
    macro_rules! make { () => { Player::new() } }
    let expanded = make!();
    let callback = || {
        let inside = get();
    };
    macro_rules! identity { ($e:expr) => { $e }; }
    let forwarded = identity!(Player::new());
    struct Defaulted<T = u8> { value: T }
    let missing = Defaulted { value: 1u8 };
    let nested_arg = Vec::<Option<_>>::new();
    nested_arg.into_iter().for_each(|_: Option<u8>| ());
    Some(())
}
fn main() {}
