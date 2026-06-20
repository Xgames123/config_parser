# config_parser

Parser and serde like derive macros for a mostly kdl compatible configuration format.

## Reason for existence

This crate is created because the main kdl library for rust currently is [knus](https://crates.io/crates/knus) which lacks a proper `#[knus(flatten)]` and also doesn't work great when your types have generics.

The main alternative for knus would have been facet-kdl but it is deprecated now.

## Features

- Fancy errors with miette
- Parses 25x than libkdl
- serde like derive syntax
- `#[config(flatten)]` Which works in almost all contexts.

## Todo

- [ ] Better documentation
- [ ] Support more of kdl

## kdl compatibility

This is the kdl syntax that is currently implemented:

```kdl

my_node properties_are_supported=#true {
    quoted-string "i am quoted"
    ident-string i_am_ident // ident strings can only contain ascii: a-z_- (which is different from the kdl spec)
    bool #true
    number 1
    hex-number 0xf
    bin-number 0b00
    float-number 0.5

    inline-comments /* inline comments are supported */ #true
}

```

## Errors

To make the fancy errors work. the "fancy" feature needs to be enabled on miette. This can be done by adding the following code to your Cargo.toml file.

`Cargo.toml`

```toml
[dependencies]
miette = { version="7.2.0", features=["fancy"] }
```

Also the source_code is not automatically attached to the error by `from_str` This can be done like this:

```rust
let parsed = config_parser::from_str(source_code).unwrap_or_else(|e| {
    panic!(
        "{:?}",
        Report::from(e).with_source_code(source_code.to_string())
    )
});
```

## Spans

`ParseConfigValue` is implemented for `parsey::Spanned<T>` so you can get span information like this:

```rust
use config_parser::Spanned; // parsey::Spanned is reexported

#[derive(ConfigNode)]
struct MyNode {
    #[config(property)]
    my_value: Spanned<String>,
}

```
