# topfew-rs: Rust implementation of Tim Bray's topfew tool

Tim Bray recently [blogged about topfew](https://www.tbray.org/ongoing/When/202x/2020/05/18/TopFew),
his Go [implementation](https://github.com/timbray/topfew/) of a tool to replace the
`awk '{print $1}' access_log | sort | uniq -c | sort -rn | head -12` pipeline.
This is a reimplementation of that tool in idiomatic stable Rust.

The initial Rust version was 2.9x faster than Tim's Go implementation;
current master appears to be around 4.6x faster thanks to some additional optimizations.
The Rust version currently has slightly fewer non-test SLOC than the Go version.
