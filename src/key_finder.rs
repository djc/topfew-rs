use anyhow::{anyhow, Error};

pub struct KeyFinder {
    keys: Option<(Vec<usize>, usize)>,
}

impl KeyFinder {
    pub fn new(keys: Option<Vec<usize>>) -> Self {
        let keys = keys.map(|mut keys| {
            keys.sort();

            let last = *keys.last().unwrap();
            let keep = (0..=last)
                .map(|i| keys.contains(&(i + 1)))
                .collect::<Vec<_>>();

            let mut offsets = Vec::new();
            let mut last = usize::MAX;
            for (idx, &k) in keep.iter().enumerate() {
                if k {
                    offsets.push(idx.wrapping_sub(last).wrapping_sub(1));
                    last = idx;
                }
            }
            (offsets, last)
        });
        KeyFinder { keys }
    }

    pub fn key<'a>(&self, record: &'a str, s: &'a mut String) -> Result<&'a str, Error> {
        let (keep, last) = match &self.keys {
            None => return Ok(record),
            Some(keep) if keep.0.len() == 0 => return Ok(record),
            Some(keep) => keep,
        };

        let mut current = 0;
        let mut iter = record.splitn(last + 2, |c| (c == ' ' || c == '\t'));
        for &offset in keep {
            match iter.nth(offset) {
                None => break,
                Some(field) => {
                    if current > 0 {
                        s.push(' ');
                    }
                    s.push_str(field);
                    current += 1;
                }
            }
        }

        if current >= keep.len() {
            Ok(s)
        } else {
            Err(anyhow!("not enough fields to make key"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key() {
        let kf = KeyFinder::new(Some(vec![1]));
        let mut s = String::new();
        assert_eq!(kf.key(TEST, &mut s).unwrap(), "92.109.155.34");

        s.clear();
        let kf = KeyFinder::new(Some(vec![7]));
        assert_eq!(kf.key(TEST, &mut s).unwrap(), "/");
    }

    const TEST: &str = "92.109.155.34 - - [09/Aug/2018:11:53:26 +0200] \"GET / HTTP/2.0\" 200 3219 \"https://www.facebook.com/\" \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36\"";
}
