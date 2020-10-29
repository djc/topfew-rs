use std::str::FromStr;

use anyhow::{anyhow, Error};
use structopt::StructOpt;

use topfew::{top_few_from_stream, KeyFinder};

fn main() -> Result<(), Error> {
    let options = Options::from_args();
    if options.num < 1 {
        return Err(anyhow!(
            "--num needs to be 1 or larger, got {}",
            options.num
        ));
    }
    let kf = KeyFinder::new(Some(options.fields.indices));
    let top_list = top_few_from_stream(options.file.into(), &kf, options.num)?;
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
