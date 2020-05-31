use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

use anyhow::Context;

/// Create an Iterator over chunks of lines of a file.
///
/// The last line in a chunk potentially reads over the chunk byte boundary to find the line end.
/// In the same way the first line searches the line end.
pub fn chunks(path: PathBuf) -> anyhow::Result<Chunks<BufReader<File>>> {
    let size = File::open(&path)?.metadata()?.len();
    let it = (0..usize::MAX).map(move |_| {
        File::open(&path)
            .map(BufReader::new)
            .with_context(|| "Failed")
    });

    let cpus = num_cpus::get() as u64;
    let chunk_size = MAX_CHUNK_SIZE.min(size / cpus / 10).max(MIN_CHUNK_SIZE);
    Ok(Chunks {
        chunk_data: Box::new(it),
        position: 0,
        starts: Box::new(split(chunk_size as u64, size)),
        chunk_size,
        size,
    })
}

pub struct Chunks<T: BufRead + Seek> {
    chunk_data: Box<dyn Iterator<Item = anyhow::Result<T>> + Send>,
    position: u64,
    starts: Box<dyn Iterator<Item = u64>>,
    chunk_size: u64,
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
            Chunk::new(f, self.chunk_size, self.position, start, self.size).ok()?;
        self.position = position;
        Some(chunk)
    }
}

#[derive(Debug)]
pub struct Chunk<C> {
    lines: std::io::Lines<C>,
    position: u64,
    end: u64,
}

impl<C> Chunk<C> {
    pub fn new(
        mut chunk: C,
        chunk_size: u64,
        mut position: u64,
        start: u64,
        size: u64,
    ) -> anyhow::Result<(Self, u64)>
    where
        C: Seek + BufRead,
    {
        let skip = if position > start {
            true
        } else if start != position {
            chunk.seek(SeekFrom::Start(start - 1))?;
            let mut buf = [0 as u8; 1];
            if let Ok(1) = chunk.read(&mut buf) {
                buf[0] != b'\n'
            } else {
                false
            }
        } else {
            false
        };

        chunk.seek(SeekFrom::Start(start))?;
        position = if skip {
            let mut skip_leader = String::new();
            let _ = chunk.read_line(&mut skip_leader)?;
            start + skip_leader.len() as u64
        } else {
            start
        };
        let lines = chunk.lines();
        let c = Self {
            lines,
            position,
            end: size.min(start + chunk_size),
        };
        Ok((c, position))
    }
}

impl<T> Iterator for Chunk<T>
where
    T: Seek + BufRead,
{
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.position >= self.end {
            return None;
        }
        let l = self.lines.next()?.ok()?;
        self.position += l.len() as u64 + 1;
        Some(l)
    }
}

fn split(chunk_size: u64, size: u64) -> impl Iterator<Item = u64> {
    let e = if chunk_size == 0 {
        0
    } else {
        size / chunk_size + 1.min(size % chunk_size)
    };
    (0..e).map(move |i| i * chunk_size)
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
        fn s(chunk_size: u64, size: u64) -> TestResult {
            let actual = split(chunk_size, size).collect::<Vec<_>>();
            let l = actual.len() as u64;
            if chunk_size != 0 {
                println!(
                    "{:?} {} {} {}",
                    actual,
                    l,
                    size / chunk_size + 1,
                    size / chunk_size
                );
            }
            TestResult::from_bool(
                (chunk_size == 0 && l == 0)
                    || ((0..(actual.len() as i64 - 1))
                        .all(|i| actual[i as usize + 1] - actual[i as usize] == chunk_size)
                        && (if size % chunk_size == 0 {
                            size / chunk_size == l
                        } else {
                            size / chunk_size == l - 1
                        })),
            )
        }
        quickcheck::QuickCheck::new().quickcheck(s as fn(_, _) -> TestResult);
    }

    fn mem_chunks<'a>(mem: Vec<u8>, chunk_size: u64, size: u64) -> Chunks<impl BufRead + Seek> {
        let it = (0..usize::MAX).map(move |_| Ok(Cursor::new(mem.clone())));
        Chunks {
            chunk_data: Box::new(it),
            position: 0,
            starts: Box::new(split(chunk_size as u64, size)),
            chunk_size,
            size,
        }
    }

    #[test]
    fn test_chunks() {
        fn test_split_buf(i: Vec<String>, chunk_size: u64) -> TestResult {
            fn t(b: String, chunk_size: u64) -> anyhow::Result<()> {
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
