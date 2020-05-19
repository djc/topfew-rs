use std::fs::File;
use std::str::FromStr;

use anyhow::Error;
use structopt::StructOpt;

use topfew::{top_few_from_stream, KeyFinder};

fn main() -> Result<(), Error> {
    let options = Options::from_args();
    let file = File::open(options.fname)?;
    let kf = KeyFinder::new(Some(options.fields.indices))?;
    let top_list = top_few_from_stream(file, &kf, options.few)?;
    for kc in top_list {
        println!("{} {}", kc.count, kc.key);
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "topfew")]
struct Options {
    #[structopt(long, default_value = "10")]
    few: usize,
    #[structopt(long)]
    fields: FieldSpec,
    fname: String,
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
