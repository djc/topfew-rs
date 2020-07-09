pub struct Chunk<'a> {
    chunk_size: u64,
    state: State<'a>,
}

impl<'a> Chunk<'a> {
    pub fn new(bytes: &'a [u8], first: bool, chunk_size: u64) -> Self {
        Self {
            chunk_size,
            state: State::Start { bytes, first },
        }
    }
}

impl<'a> Iterator for Chunk<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let (lines, pos) = match self.state {
            State::Start { mut bytes, first } => {
                let mut position = 0;
                if !first {
                    while bytes[0] != b'\n' {
                        bytes = &bytes[1..];
                        position += 1;
                    }
                    bytes = &bytes[1..];
                    position += 1;
                }
                let lines = std::str::from_utf8(bytes).unwrap().lines();
                self.state = State::Running { lines, position };
                match &mut self.state {
                    State::Running { lines, position } => (lines, position),
                    _ => unreachable!(),
                }
            }
            State::Running {
                ref mut lines,
                ref mut position,
            } => (lines, position),
            State::End => return None,
        };

        if *pos > self.chunk_size {
            self.state = State::End;
            return None;
        }

        let l = lines.next()?;
        *pos += l.len() as u64 + 1;
        Some(l)
    }
}

enum State<'a> {
    Start {
        bytes: &'a [u8],
        first: bool,
    },
    Running {
        lines: std::str::Lines<'a>,
        position: u64,
    },
    End,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use std::io::Cursor;

    #[test]
    fn test_chunks() {
        fn test_split_buf(i: Vec<String>, chunk_size: u64) -> TestResult {
            fn t(b: String, chunk_size: u64) -> anyhow::Result<()> {
                let bytes = b.as_bytes().to_owned();
                let size = bytes.len() as u64;
                let source = MemorySource { bytes };

                let chunks: Vec<_> = Chunker::new(source, size, chunk_size)
                    .into_iter()
                    .map(|i| i.collect::<Vec<_>>().join("\n"))
                    .collect();

                let r = regex::Regex::new(r"\s+")?;
                let e = r.replace_all(&b, " ");
                let cs = chunks.join(" ");
                let a = r.replace_all(&cs, " ");
                if e.trim() == a.trim() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Expected >{}< Actual >{}<",
                        e.trim(),
                        a.trim()
                    ))
                }
            }

            let filter = regex::Regex::new(r"\W").unwrap();
            if chunk_size < 1 || i.iter().any(|s| filter.is_match(&s)) {
                TestResult::discard()
            } else if let Err(e) = t(i.join("\n"), chunk_size) {
                TestResult::error(format!("{:?}", e))
            } else {
                TestResult::from_bool(true)
            }
        }
        quickcheck::QuickCheck::new()
            .max_tests(300)
            .quickcheck(test_split_buf as fn(_, _) -> TestResult);
    }
}
