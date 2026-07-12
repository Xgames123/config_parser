use std::hint::black_box;
extern crate test;
use test::Bencher;

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
#[config(node_name("entrypoint"))]
struct PackageEntrypoint {
    #[knus(argument)]
    #[config(argument)]
    entrypoint: String,
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
#[config(node_name("name"))]
struct PackageName {
    #[knus(argument)]
    #[config(argument)]
    name: String,
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
enum PackageNode {
    Entrypoint(PackageEntrypoint),
    Name(PackageName),
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
struct CargoBuilder {
    #[knus(property)]
    bin: String,
    #[knus(property)]
    dest: String,
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
enum PackageContentItem {
    Cargo(CargoBuilder),
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
#[config(node_name("content"))]
struct PackageContent {
    #[knus(children)]
    #[config(children)]
    content_items: Vec<PackageContentItem>,
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
struct Package {
    #[knus(child)]
    #[config(child)]
    content: PackageContent,

    #[knus(children)]
    #[config(children)]
    nodes: Vec<PackageNode>,
}

#[derive(knus::Decode, config_parser::ConfigNode, Debug, PartialEq)]
struct File {
    #[knus(child)]
    #[config(child)]
    package: Package,
}

fn derive_bench_content() -> &'static str {
    r#"
package {
  name "starpack"
  entrypoint "/usr/bin/starpack"

  content {
      cargo bin="starpack" dest="${binary_path}"
  }
}
"#
}

fn derive_bench_match(file: File) {
    assert_eq!(
        file,
        File {
            package: Package {
                content: PackageContent {
                    content_items: vec![PackageContentItem::Cargo(CargoBuilder {
                        bin: "starpack".into(),
                        dest: "${binary_path}".into()
                    })]
                },
                nodes: vec![
                    PackageNode::Name(PackageName {
                        name: "starpack".into()
                    }),
                    PackageNode::Entrypoint(PackageEntrypoint {
                        entrypoint: "/usr/bin/starpack".into()
                    })
                ]
            }
        }
    )
}

#[bench]
fn knus_derive(b: &mut Bencher) {
    let content = derive_bench_content();
    b.iter(|| derive_bench_match(black_box(knus::parse::<File>("file.kdl", content).unwrap())));
}
#[bench]
fn config_parser_derive(b: &mut Bencher) {
    let content = derive_bench_content();
    b.iter(|| derive_bench_match(black_box(config_parser::from_str::<File>(content).unwrap())));
}
