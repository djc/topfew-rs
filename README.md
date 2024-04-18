# topfew-rs: Rust implementation of Tim Bray's topfew tool

Tim Bray [blogged about topfew](https://www.tbray.org/ongoing/When/202x/2020/05/18/TopFew),
his Go [implementation](https://github.com/timbray/topfew/) of a tool to replace the
`awk '{print $1}' access_log | sort | uniq -c | sort -rn | head -12` pipeline.
This is a reimplementation of that tool in idiomatic stable Rust.

The initial Rust version was 2.9x faster than Tim's Go implementation;
current main appears to be around 6.7x faster thanks to some additional optimizations.
The Rust version currently has slightly fewer non-test SLOC than the Go version.
Thanks to [Thomas Andreas Jung](https://github.com/blob79) for contributing substantial
performance improvements.

```
Usage: tf [OPTIONS] --fields <FIELDS> <FILE>

Arguments:
  <FILE>  File to search

Options:
  -f, --fields <FIELDS>  Fields to use as part of the line's key
  -n, --num <NUM>        Top number of matches to show [default: 10]
  -e, --regex <REGEX>    Regular expression used to split lines into fields [default: "[ \\t]"]
  -h, --help             Print help
```

If you have the Rust toolchain installed, you can install it with `cargo install topfew`.
