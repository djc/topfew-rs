use std::str::FromStr;

use anyhow::{anyhow, Error};
use clap::Parser;
use regex::Regex;

use topfew::{top_few_from_stream, KeyFinder};

fn main() -> Result<(), Error> {
    let options = Options::parse();
    if options.num < 1 {
        return Err(anyhow!(
            "--num needs to be 1 or larger, got {}",
            options.num
        ));
    }
    let sep = Regex::new(&options.regex)?;
    let kf = KeyFinder::new(Some(options.fields.indices), sep)?;
    let top_list = top_few_from_stream(options.file.into(), &kf, options.num)?;
    for kc in top_list {
        println!("{} {}", kc.count, kc.key);
    }
    Ok(())
}

#[derive(Debug, Parser)]
#[clap(name = "topfew")]
struct Options {
    /// Fields to use as part of the line's key
    #[clap(long, short)]
    fields: FieldSpec,
    /// Top number of matches to show
    #[clap(long, short, default_value = "10")]
    num: usize,
    /// Regular expression used to split lines into fields
    #[clap(long, short = 'e', default_value = "[ \\t]")]
    regex: String,
    /// File to search
    file: String,
}

#[derive(Debug, Clone)]
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

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
