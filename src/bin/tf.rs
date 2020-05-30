use std::path::Path;
use std::str::FromStr;

use anyhow::Error;
use regex::Regex;
use structopt::StructOpt;

use topfew::{top_few_from_stream, KeyFinder};

fn main() -> Result<(), Error> {
    let options = Options::from_args();
    let sep = Regex::new(&options.regex)?;
    let kf = KeyFinder::new(Some(options.fields.indices), sep)?;
    let top_list = top_few_from_stream(&Path::new(&options.file), &kf, options.num)?;
    for kc in top_list {
        println!("{} {}", kc.count, kc.key);
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "topfew")]
struct Options {
    /// Fields to use as part of the line's key
    #[structopt(long, short)]
    fields: FieldSpec,
    /// Top number of matches to show
    #[structopt(long, short = "n", default_value = "10")]
    num: usize,
    /// Regular expression used to split lines into fields
    #[structopt(long, short = "e", default_value = "[ \\t]")]
    regex: String,
    /// File to search
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
