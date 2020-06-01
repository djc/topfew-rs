use anyhow::{anyhow, Error};
use regex::Regex;

pub struct KeyFinder {
    keys: Option<(usize, usize, Vec<bool>)>,
    sep: Regex,
}

impl KeyFinder {
    pub fn new(keys: Option<Vec<usize>>, sep: Regex) -> Result<Self, Error> {
        let keys = keys.map(|mut keys| {
            keys.sort();

            let last = *keys.last().unwrap();
            (
                keys.len(),
                last,
                (0..=last)
                    .map(|i| keys.contains(&(i + 1)))
                    .collect::<Vec<_>>(),
            )
        });
        Ok(KeyFinder { keys, sep })
    }

    pub fn key<'a>(&self, record: &'a str, s: &'a mut String) -> Result<&'a str, Error> {
        let (num, last, keep) = match &self.keys {
            None => return Ok(record),
            Some((num, _, _)) if *num == 0 => return Ok(record),
            Some((num, last, keep)) => (num, last, keep),
        };

        let mut fields = keep
            .iter()
            .zip(self.sep.splitn(record, last + 2))
            .filter_map(|(keep, field)| if *keep { Some(field) } else { None });

        if *num == 1 {
            return match fields.next() {
                Some(f) => Ok(f),
                None => Err(anyhow!("not enough fields to make key")),
            };
        }

        let mut found = 0;
        for f in fields {
            s.push(' ');
            s.push_str(f);
            found += 1;
        }

        if found == *num {
            Ok(s)
        } else {
            Err(anyhow!("not enough fields to make key"))
        }
    }
}
