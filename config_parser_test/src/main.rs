#![cfg_attr(test, feature(test))]
use std::{
    io::{Read, stdin},
    time::Instant,
};

use config_parser::parse::Document;
use miette::Report;

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

#[cfg(test)]
mod bench {
    use std::hint::black_box;
    extern crate test;
    use test::Bencher;

    fn content() -> &'static str {
        r#"
package {
  name "starpack"
  entrypoint "/usr/bin/starpack"
  port 8080

  target "musl+linux/x86_64"
  target "linux/arm/v7" if="build.docker"

  dep "libssh2" optional=#true if="!target.windows"
  dep "docker" if="!target.windows"

  build-dep "openssl-dev" "openssl-libs-static" "musl" "musl-dev" "gcc"
  content {
    // copy "LICENCE.md" dest="/usr/share/licenses/diststar/" if="target.unix"
    // copy "LICENCE.md" dest="%ProgramFiles%/diststar/" if="!target.unix"
    cargo bin="starpack" dest="${binary_path}"
  }


  // env PATH "${binary_path}" if="windows"
}

build {
  docker push="myregistry"
  // deb push="ldeveuorg:/tmp/debian"
  // winget push="ldeveuorg:/tmp/winget"
  // pkgbuild
}

push {
  ssh-remote "ldev@192.168.1.69" after-cmd="echo 'files have been transferred'" if="build.docker"
  github-release if="build.debian"
  winget if="build.winget"
  aur "starpack-bin" "starpack-git" "starpack" if="build.pkgbuild"
}
"#
    }

    #[bench]
    fn libkdl_impl(b: &mut Bencher) {
        let content = content();
        b.iter(|| black_box(kdl::KdlDocument::parse_v2(&content).unwrap()))
    }

    #[bench]
    fn my_impl(b: &mut Bencher) {
        let content = content();
        b.iter(|| black_box(config_parser::parse::Document::from_str(&content).unwrap()))
    }
}
