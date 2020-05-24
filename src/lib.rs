use std::io::BufRead;

mod counter;
mod key_finder;

pub use counter::{Counter, KeyCount};
pub use key_finder::KeyFinder;

pub fn top_few_from_stream<T: BufRead>(
    reader: T,
    kf: &KeyFinder,
    num: usize,
) -> Result<Vec<KeyCount>, anyhow::Error> {
    let mut counter = Counter::new(num);
    let mut s = String::new();
    for ln in reader.lines() {
        s.clear();
        let ln = ln?;
        if let Ok(key) = kf.key(&ln, &mut s) {
            counter.add(key)
        }
    }

    Ok(counter.top())
}
