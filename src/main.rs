use std::fs::File;
use std::io::Cursor;
use std::str::FromStr;

use anyhow::Error;
use memmap::MmapOptions;
use structopt::StructOpt;

use topfew::{top_few_from_stream, KeyFinder};

fn main() -> Result<(), Error> {
    let options = Options::from_args();
    let file = File::open(options.file)?;
    let bytes = unsafe { MmapOptions::new().map(&file)? };
    let kf = KeyFinder::new(Some(options.fields.indices))?;
    let top_list = top_few_from_stream(Cursor::new(bytes), &kf, options.num)?;
    for kc in top_list {
        println!("{} {}", kc.count, kc.key);
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "topfew")]
struct Options {
    #[structopt(long, short)]
    fields: FieldSpec,
    #[structopt(long, short = "n", default_value = "10")]
    num: usize,
    file: String,
}

#[derive(Debug)]
struct FieldSpec {
    indices: Vec<usize>,
}

impl FromStr for FieldSpec {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut indices = Vec::new();
        for f in s.split(',') {
            indices.push(usize::from_str(f)?);
        }
        Ok(FieldSpec { indices })
    }
}
