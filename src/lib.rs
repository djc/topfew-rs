use std::path::Path;

use rayon::prelude::*;

mod chunks;
mod counter;
mod key_finder;

pub use chunks::chunks;
pub use counter::{Counter, KeyCount};
pub use key_finder::KeyFinder;

pub fn top_few_from_stream(
    path: &Path,
    kf: &KeyFinder,
    num: usize,
) -> anyhow::Result<Vec<KeyCount>> {
    let total = chunks(path)?
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|reader| {
            let mut counter = Counter::new(None);
            let mut s = String::new();
            for ln in reader {
                s.clear();
                if let Ok(key) = kf.key(&ln, &mut s) {
                    counter.add(key, 1)
                }
            }
            counter
        })
        .fold(|| Counter::new(None), sum)
        .reduce(|| Counter::new(Some(num)), sum);

    Ok(total.top())
}

fn sum(mut l: Counter, r: Counter) -> Counter {
    l.merge(r);
    l
}
