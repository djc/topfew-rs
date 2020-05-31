use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

use anyhow::Context;

/// Create an Iterator over chunks of lines of a file.
///
/// The last line in a chunk potentially reads over the chunk byte boundary to find the line end.
/// In the same way the first line searches the line end.
pub fn chunks(path: &Path) -> anyhow::Result<Chunks<BufReader<File>>> {
    let path = path.to_path_buf();
    let size = File::open(&path)?.metadata()?.len();
    let it = (0..usize::MAX).map(move |_| {
        File::open(&path)
            .map(BufReader::new)
            .with_context(|| "Failed")
    });

    let cpus = num_cpus::get() as u64;
    let chunk_size = MAX_CHUNK_SIZE.min(size / cpus / 10).max(MIN_CHUNK_SIZE) as usize;
    Ok(Chunks {
        chunk_data: Box::new(it),
        current: 0,
        starts: Box::new(split(chunk_size as u64, size)),
        chunk_size,
        size,
    })
}

pub struct Chunks<T: BufRead + Seek> {
    chunk_data: Box<dyn Iterator<Item = anyhow::Result<T>> + Send>,
    current: u64,
    starts: Box<dyn Iterator<Item = u64>>,
    chunk_size: usize,
    size: u64,
}

impl<T> Iterator for Chunks<T>
where
    T: BufRead + Seek,
{
    type Item = Chunk<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.starts.next()?;
        let f = self.chunk_data.next()?.ok()?;
        let (chunk, position) =
            Chunk::new(f, self.chunk_size, self.current, start, self.size).ok()?;
        self.current = position;
        Some(chunk)
    }
}

#[derive(Debug)]
pub struct Chunk<C> {
    lines: std::io::Lines<C>,
    pos: u64,
    end: u64,
}

impl<C> Chunk<C> {
    pub fn new(
        mut c: C,
        chunk: usize,
        mut current: u64,
        start: u64,
        size: u64,
    ) -> anyhow::Result<(Self, u64)>
    where
        C: Seek + BufRead,
    {
        let skip = if current > start {
            true
        } else if start != current {
            c.seek(SeekFrom::Start(start - 1))?;
            let mut buf = [0 as u8; 1];
            if let Ok(1) = c.read(&mut buf) {
                buf[0] != b'\n'
            } else {
                false
            }
        } else {
            false
        };

        c.seek(SeekFrom::Start(start))?;
        current = if skip {
            let mut skip_leader = String::new();
            let _ = c.read_line(&mut skip_leader)?;
            start + skip_leader.len() as u64
        } else {
            start
        };
        let lines = c.lines();
        let c = Self {
            lines,
            pos: current,
            end: size.min(start + chunk as u64),
        };
        Ok((c, current))
    }
}

impl<T> Iterator for Chunk<T>
where
    T: Seek + BufRead,
{
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.pos >= self.end {
            return None;
        }
        let l = self.lines.next()?.ok()?;
        self.pos += l.len() as u64 + 1;
        Some(l)
    }
}

fn split(chunk: u64, size: u64) -> impl Iterator<Item = u64> {
    let e = if chunk == 0 {
        0
    } else {
        size / chunk + 1.min(size % chunk)
    };
    (0..e).map(move |i| i * chunk)
}

const MIN_CHUNK_SIZE: u64 = 512 * 1024;
const MAX_CHUNK_SIZE: u64 = 64 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use std::io::Cursor;

    #[test]
    fn test_split() {
        fn s(chunk: u64, size: u64) -> TestResult {
            let actual = split(chunk, size).collect::<Vec<_>>();
            let l = actual.len() as u64;
            if chunk != 0 {
                println!("{:?} {} {} {}", actual, l, size / chunk + 1, size / chunk);
            }
            TestResult::from_bool(
                (chunk == 0 && l == 0)
                    || ((0..(actual.len() as i64 - 1))
                        .all(|i| actual[i as usize + 1] - actual[i as usize] == chunk)
                        && (if size % chunk == 0 {
                            size / chunk == l
                        } else {
                            size / chunk == l - 1
                        })),
            )
        }
        quickcheck::QuickCheck::new().quickcheck(s as fn(_, _) -> TestResult);
    }

    fn mem_chunks<'a>(mem: Vec<u8>, chunk_size: usize, size: u64) -> Chunks<impl BufRead + Seek> {
        let it = (0..usize::MAX).map(move |_| Ok(Cursor::new(mem.clone())));
        Chunks {
            chunk_data: Box::new(it),
            current: 0,
            starts: Box::new(split(chunk_size as u64, size)),
            chunk_size,
            size,
        }
    }

    #[test]
    fn test_chunks() {
        fn test_split_buf(i: Vec<String>, chunk_size: usize) -> TestResult {
            fn t(b: String, chunk_size: usize) -> anyhow::Result<()> {
                let chunks: Vec<_> =
                    mem_chunks(b.as_bytes().to_owned(), chunk_size, b.len() as u64)
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
                return TestResult::discard();
            }
            if let Err(e) = t(i.join("\n"), chunk_size) {
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
