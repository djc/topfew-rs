use std::borrow::Cow;

use anyhow::{anyhow, Error};
use regex::Regex;

pub struct KeyFinder {
    keys: Option<Vec<usize>>,
    sep: Regex,
}

impl KeyFinder {
    pub fn new(mut keys: Option<Vec<usize>>) -> Result<Self, Error> {
        if let Some(keys) = &mut keys {
            keys.iter_mut().for_each(|v| *v -= 1);
        }
        Ok(KeyFinder {
            keys,
            sep: Regex::new("\\s+")?,
        })
    }

    pub fn key<'a>(&self, record: &'a str) -> Result<Cow<'a, str>, Error> {
        let keys = match &self.keys {
            None => return Ok(record.into()),
            Some(keys) if keys.is_empty() => return Ok(record.into()),
            Some(keys) => keys,
        };

        let fields = self.sep.splitn(record, keys[keys.len() - 1] + 2).collect::<Vec<_>>();
        if fields.len() <= keys[keys.len() - 1] {
            return Err(anyhow!("not enough fields to make key"));
        }

        if keys.len() == 1 {
            return Ok(fields[keys[0]].into())
        }

        let mut s = String::new();
        s.push_str(fields[keys[0]]);
        for key in keys.iter().skip(1) {
            s.push(' ');
            s.push_str(fields[*key]);
        }
        Ok(s.into())
    }
}
