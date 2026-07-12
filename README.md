# config_parser

A fast, mostly-KDL-compatible configuration parser for Rust with serde-style derive macros.

`config_parser` focuses on ergonomic configuration parsing, good diagnostics, and features that are difficult to express with existing KDL libraries, such as robust flattening support and generic types.

## Why?

The existing KDL ecosystem has a few limitations:

- [`knus`](https://crates.io/crates/knus) does not currently provide a general-purpose `#[knus(flatten)]`.
- Generic types can be awkward to work with in derive-based parsers.
- [`facet-kdl`](https://crates.io/crates/facet-kdl) has been deprecated.

`config_parser` exists to provide a more ergonomic alternative while remaining largely compatible with KDL syntax.

## Features

- Fast parser implementation (~25× faster than `libkdl` and `knus` on my machine)
- Serde-style derive macros
- `#[config(flatten)]` support in almost all contexts
- Fancy errors powered by `miette`
- Generics support see `config_parser/examples/full.rs`
- Source span support

## Roadmap

- [ ] Improve documentation and examples
- [ ] Expand KDL compatibility

## Supported KDL Syntax

The following syntax is currently supported:

```kdl
my_node properties_are_supported=#true {
    quoted-string "i am quoted" // ident strings can only contain ascii: a-z_- (which is different from the kdl spec)
    ident-string i_am_ident

    bool #true
    number 1
    hex-number 0xf
    bin-number 0b00
    float-number 0.5

    inline-comments /* inline comments are supported */ #true
}
```

## Error Reporting

To get the fancy error messages, enable the `fancy` feature of `miette` by adding the following to your crates `Cargo.toml` (Only do this for the root crate):

**Cargo.toml**

```toml
[dependencies]
miette = { version = "7.2.0", features = ["fancy"] }
```

`config_parser::from_str` does not automatically attach source code to errors. To display fancy diagnostics, attach the source manually:

```rust
use miette::Report;

#[derive(config_parser::ConfigNode)]
struct MyNode;

let source_code = "";

let parsed : MyNode = config_parser::from_str(source_code).unwrap_or_else(|e| {
    panic!(
        "{:?}",
        Report::from(e).with_source_code(source_code.to_string())
    )
});
```

## Span Information

`ParseConfigValue` and `ParseConfigNode` are implemented for `starryparse::Spanned<T>` (re-exported as `config_parser::Spanned`), allowing easy access span information:

```rust
use config_parser::{Spanned, ConfigNode};

#[derive(ConfigNode)]
struct ChildNode;

#[derive(ConfigNode)]
struct MyNode {
    #[config(property)]
    my_value: Spanned<String>,

    #[config(child)]
    my_child:  Spanned<ChildNode>
}

```

## Benchmarks

The speed of configuration parsing doesn't matter in most cases except for if the configuration file is very large and needs to be read at startup by command line tools.

The following benchmark measures parsing the same kdl file using both `libkdl` and `config_parser`.

> **Note:** This is not a perfectly apples-to-apples comparison. `libkdl` supports additional features, including document editing capabilities, which may affect performance.

| Crate           |             Mean Time |
| --------------- | --------------------: |
| `libkdl`        | 410,071 ns ± 6,903 ns |
| `config_parser` |  16,219 ns ± 1,275 ns |

This benchmark measures parsing the same configuration using the derive macros of both `config_parser` and `knus`.

> **Note:** This is not a completely fair comparison because `knus` supports more kdl features than `config_parser`.

| Crate           |                Mean Time |
| --------------- | -----------------------: |
| `config_parser` |   3,079.90 ns ± 16.96 ns |
| `knus`          | 76,024.60 ns ± 364.10 ns |

The source of both benchmarks is located at `config_parser_test/main.rs`.

To reproduce the results:

```bash
cargo +nightly bench
```

## Licence

Copyright 2026 S.v.e.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

[http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
