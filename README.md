# AutoArgs [![Build Status](https://travis-ci.org/droundy/auto-args.svg?branch=master)](https://travis-ci.org/droundy/auto-args) [![](https://img.shields.io/crates/v/auto-args.svg)](https://crates.io/crates/auto-args) [![](https://docs.rs/auto-args/badge.svg)](https://docs.rs/auto-args)

Parse command line arguments by defining a struct.  It combines
[clap](https://crates.io/crates/clap) with custom derive.

The basic idea is that you define a type that represents the
information you want on the command-line from the person running your
program, and `derive(AutoArgs)` on that type, and then call
`YourType::from_args()` to find out what your user gave you.

## Differences from ClapMe

AutoArgs is essentially equivalent to ClapMe.  It only differs in
implementation, speed of compilation, help messages, and error messages.

## Differences from structopt

[StructOpt](https://docs.rs/structopt) is a tool that serves the same
function as `auto-args`, but with a different philosophy.  StructOpt
strives to be as expressive as clap, and to enable all features clap
does, and with a tightly coupled API.  It does this by extensive use
of attributes such as `#[structopt(long = "long-name")]`, which define
the behavior and property of each flag.  This makes it powerful, but
also rather verbose to use.

In contrast, AutoArgs does not (yet?) support any attributes, and
determines all behavior directly from your type.  This means that you
don't repeat yourself, but also means that you have less fine-grained
control over your interface, and some interfaces may be well-nigh
impossible.

You *can* implement the `AutoArgs` trait manually, which does make it
possible (and not even that painful) to create a different
command-line for a given type, but this is not the intended way to use
`AutoArgs`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

