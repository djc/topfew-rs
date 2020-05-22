use std::borrow::Cow;

use anyhow::{anyhow, Error};
use regex::Regex;
use std::boxed::Box;

pub struct KeyFinder {
    keys: Option<Vec<usize>>,
    sep: Regex,
}

impl KeyFinder {
    pub fn new(mut keys: Option<Vec<usize>>) -> Result<Self, Error> {
        if let Some(keys) = &mut keys {
            keys.iter_mut().for_each(|v| *v -= 1);
            keys.sort();
        }
        Ok(KeyFinder {
            keys,
            sep: Regex::new("\\s+")?,
        })
    }

    pub fn key<'a>(&self, record: &'a str) -> Result<Cow<'a, str>, Error> {
        let kkeys = match &self.keys {
            None => return Ok(record.into()),
            Some(keys) if keys.is_empty() => return Ok(record.into()),
            Some(keys) => keys,
        };

        let mut keys = kkeys.iter();
        let mut fields: Box<dyn Iterator<Item = _>> = Box::new(
            self.sep
                .splitn(record, kkeys[kkeys.len() - 1] + 2)
                .skip(*(keys.next().unwrap())),
        );
        let mut s;
        if let Some(field) = fields.next() {
            s = Cow::from(field);
        } else {
            return Err(anyhow!("not enough fields to make key"));
        }

        let mut last = 0;
        for key in keys {
            fields = Box::new(fields.skip(key - last));
            last = key + 1;
            if let Some(field) = fields.next() {
                let s1 = s.to_mut();
                s1.push(' ');
                s1.push_str(field);
            } else {
                return Err(anyhow!("not enough fields to make key"));
            }
        }

        Ok(s)
    }
}
