# topfew-rs: Rust implementation of Tim Bray's topfew tool

Tim Bray recently [blogged about topfew](https://www.tbray.org/ongoing/When/202x/2020/05/18/TopFew),
his Go [implementation](https://github.com/timbray/topfew/) of a tool to replace the
`awk '{print $1}' access_log | sort | uniq -c | sort -rn | head -12` pipeline.
This is a reimplementation of that tool in idiomatic stable Rust.

The initial Rust version was 2.9x faster than Tim's Go implementation;
current master appears to be around 4.6x faster thanks to some additional optimizations.
The Rust version currently has slightly fewer non-test SLOC than the Go version.

```
topfew 0.1.0

USAGE:
    tf [OPTIONS] <file> --fields <fields>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --fields <fields>    Fields to use as part of the line's key
    -n, --num <num>          Top number of matches to show [default: 10]
    -e, --regex <regex>      Regular expression used to split lines into fields [default: \s+]

ARGS:
    <file>    File to search
```

If you have the Rust toolchain installed, you can install it with `cargo install topfew`.
