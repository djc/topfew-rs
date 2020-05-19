use std::io::{BufRead, BufReader, Read};

mod key_finder;
mod counter;

pub use key_finder::KeyFinder;
pub use counter::{Counter, KeyCount};

pub fn top_few_from_stream<T: Read>(reader: T, kf: &KeyFinder, size: usize) -> Result<Vec<KeyCount>, anyhow::Error> {
    let reader = BufReader::new(reader);
    let mut counter = Counter::new(size);
    for ln in reader.lines() {
        let ln = ln?;
        if let Ok(key) = kf.key(&ln) {
            counter.add(key)
        }
    }

    Ok(counter.top())
}
