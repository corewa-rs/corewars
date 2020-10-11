# corewars

[![Latest Crates.io Release](https://img.shields.io/crates/v/corewars?label=corewars&logo=rust)](https://crates.io/crates/corewars)
[![Latest Github Release](https://img.shields.io/github/v/release/corewa-rs/corewars?label=Release&include_prereleases&logo=github)](https://github.com/corewa-rs/corewars/releases)
[![Build Status](https://img.shields.io/github/workflow/status/corewa-rs/corewars/ci/develop)](https://github.com/corewa-rs/corewars/actions)

A CLI interface for playing the classic programming battle game
[Core Wars](http://www.koth.org/index.html).

See the [website](https://corewa.rs) or the [Github repo](https://github.com/corewa-rs/corewars) for more details.

## Usage

```txt
Parse, assemble, and save Redcode files

USAGE:
    corewars <input-file> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <input-file>    Input file; use "-" to read from stdin

SUBCOMMANDS:
    dump    Save/print a program in "load file" format
    help    Prints this message or the help of the given subcommand(s)
```

### `dump` Usage

```txt
Save/print a program in "load file" format

USAGE:
    corewars <input-file> dump [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
    -E, --no-expand    Whether labels, expressions, macros, etc. should be resolved and expanded in the output
    -V, --version      Prints version information

OPTIONS:
    -o, --output-file <output-file>    Output file; defaults to stdout ("-") [default: -]
```
