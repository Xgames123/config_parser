# config_parser

A fast, mostly-KDL-compatible configuration parser for Rust with serde-style derive macros.

`config_parser` focuses on ergonomic configuration parsing, good diagnostics, and features that are difficult to express with existing KDL libraries, such as robust flattening support and generic types.

## Why?

The existing KDL ecosystem has a few limitations:

- [`knus`](https://crates.io/crates/knus) does not currently provide a general-purpose `#[knus(flatten)]` equivalent.
- Generic types can be awkward to work with in derive-based parsers.
- [`facet-kdl`](https://crates.io/crates/facet-kdl) has been deprecated.

`config_parser` exists to provide a more ergonomic alternative while remaining largely compatible with KDL syntax.

## Features

- Fast parser implementation (~25× faster than `libkdl` on my machine)
- Serde-style derive macros
- `#[config(flatten)]` support in most contexts
- Fancy errors powered by `miette`
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

For enhanced diagnostics, enable the `fancy` feature of `miette`:

**Cargo.toml**

```toml
[dependencies]
miette = { version = "7.2.0", features = ["fancy"] }
```

`config_parser::from_str` does not automatically attach source code to errors. To display fancy diagnostics, attach the source manually:

```rust
use miette::Report;

let parsed = config_parser::from_str(source_code).unwrap_or_else(|e| {
    panic!(
        "{:?}",
        Report::from(e).with_source_code(source_code.to_string())
    )
});
```

## Span Information

`ParseConfigValue` is implemented for `parsey::Spanned<T>` (re-exported as `config_parser::Spanned`), allowing easy access span information:

```rust
use config_parser::Spanned;

#[derive(ConfigNode)]
struct MyNode {
    #[config(property)]
    my_value: Spanned<String>,
}
```

## Benchmarks

The following benchmark measures parsing the same configuration file using both `libkdl` and `config_parser`. the source of the benchmark is located at `config_parser_test/main.rs`

> **Note:** This is not a perfectly apples-to-apples comparison. `libkdl` supports additional features, including document editing capabilities, which may affect performance.

| Crate           |             Mean Time |
| --------------- | --------------------: |
| `libkdl`        | 410,071 ns ± 6,903 ns |
| `config_parser` |  16,219 ns ± 1,275 ns |

To reproduce the results:

```bash
cargo +nightly bench
```
