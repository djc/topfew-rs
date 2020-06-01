use std::path::PathBuf;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod chunks;
mod counter;
mod key_finder;

pub use chunks::chunks;
pub use counter::{Counter, KeyCount};
pub use key_finder::KeyFinder;

pub fn top_few_from_stream(
    path: PathBuf,
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
        .reduce(|| Counter::new(Some(num)), |l, r| l.merge(r));

    Ok(total.top())
}
