use std::borrow::Cow;
use std::collections::HashMap;


#[derive(Default)]
pub struct Counter {
    counts: HashMap<String, u64>,
    top: HashMap<String, u64>,
    threshold: u64,
    size: usize,
}

impl Counter {
    pub fn new(size: usize) -> Self {
        Self { size, ..Default::default() }
    }

    pub fn add(&mut self, key: Cow<'_, str>) {
        let count = match self.counts.get_mut(&*key) {
            Some(count) => {
                *count += 1;
                *count
            }
            None => {
                self.counts.insert((&*key).to_owned(), 1);
                1
            }
        };

        if count < self.threshold {
            return;
        }
        self.top.insert(key.into(), count);

        if self.top.len() < self.size * 2 {
            return;
        }

        let mut top_values = self.top.values().collect::<Vec<_>>();
        top_values.sort_unstable();
        let threshold = *top_values[self.size as usize - 1];
        self.threshold = threshold;
        self.top.retain(|_, v| *v > threshold);
    }

    pub fn top(&self) -> Vec<KeyCount> {
        let mut top = Vec::with_capacity(self.size);
        for (key, &count) in &self.top {
            top.push(KeyCount { count, key: key.into() });
        }

        top.sort_unstable();
        top.reverse();
        top
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct KeyCount {
    pub count: u64,
    pub key: String,
}
