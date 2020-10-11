# corewars

[![Latest Github release](https://img.shields.io/github/v/release/corewa-rs/corewars?label=Release&include_prereleases&logo=github)](https://github.com/corewa-rs/corewars/releases)
[![Build status](https://img.shields.io/github/workflow/status/corewa-rs/corewars/ci/develop)](https://github.com/corewa-rs/corewars/actions)

[![Latest corewars release](https://img.shields.io/crates/v/corewars?label=corewars&logo=rust)](https://crates.io/crates/corewars)
[![Latest corewars-core release](https://img.shields.io/crates/v/corewars-core?label=corewars-core&logo=rust)](https://crates.io/crates/corewars-core)
[![Latest corewars-parser release](https://img.shields.io/crates/v/corewars-parser?label=corewars-parser&logo=rust)](https://crates.io/crates/corewars-parser)

A Rust implementation of the classic programming battle game
[Core Wars](http://www.koth.org/index.html).

The implementation is based on [this introductory guide](http://vyznev.net/corewar/guide.html) to Redcode, as well as the [pMARS '94 reference](https://corewa.rs/pmars-redcode-94.txt) and an [annotated version](https://corewa.rs/icws94.txt) of the ICWS '94 draft.

## Cargo Crates

* [corewars](https://crates.io/crates/corewars): the binary to run Core Wars
  from the command line.
* [corewars-core](https://crates.io/crates/corewars-core): data structures and
  utilities common to other crates (such as the representation of a "core").
* [corewars-parser](https://crates.io/crates/corewars-parser): the parser used to
  read and error-check Redcode files. Output from this crate will be used as input
  for the MARS simulation itself.
