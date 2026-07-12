#![cfg_attr(test, feature(test))]
use std::{
    io::{Read, stdin},
    time::Instant,
};

use config_parser::Document;
use miette::Report;

#[cfg(test)]
mod bench;
#[cfg(test)]
mod derive_tests;
#[cfg(test)]
mod parse_tests;

fn main() {
    let mut args = std::env::args();
    args.next();
    let file = args.next().unwrap();

    let content = if file == "-" {
        let mut buf = String::new();
        stdin().read_to_string(&mut buf).unwrap();
        buf
    } else {
        std::fs::read_to_string(&file).unwrap()
    };

    let doc = messure("libkdl", || {
        kdl::KdlDocument::parse_v2(&content).unwrap_or_else(|e| panic!("{:?}", Report::from(e)))
    });
    println!("{:?}", doc);

    let doc = messure("myimpl", || {
        Document::from_str(&content).unwrap_or_else(|e| {
            panic!(
                "{:?}",
                Report::from(e).with_source_code(content.to_string())
            )
        })
    });

    println!("{:?}", doc);
}

fn messure<O>(name: &str, f: impl FnOnce() -> O) -> O {
    let instant = Instant::now();
    let output = f();
    let elapsed = instant.elapsed();
    println!(
        "parsed {} in {}ms",
        name,
        elapsed.as_nanos() as f64 * 0.000001
    );
    output
}
