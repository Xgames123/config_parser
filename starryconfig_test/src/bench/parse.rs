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
    b.iter(|| black_box(starryconfig::Document::from_str(&content).unwrap()))
}
