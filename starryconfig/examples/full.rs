use starryconfig::{ConfigNode, ParseConfigNode};
use miette::Report;
use std::path::PathBuf;

#[derive(Debug, ConfigNode, PartialEq)]
struct Author {
    #[config(argument)]
    value: String,
}

#[derive(Debug, ConfigNode, PartialEq)]
// Specify extra trait bound because else the derive will not work.
#[config(impl_where(T: ParseConfigNode<'c>))]
// The node name will be grabbed from T (Note: this will call the
// T::allowed_node_names so if T has changed node name it works. )
#[config(node_name(T))]
pub struct Named<T> {
    #[config(flatten)]
    inner: T,

    #[config(property)]
    name: String,
}

#[derive(Debug, ConfigNode, PartialEq)]
struct Package {
    #[config(child)]
    author: Author,

    // Put builders at the bottom because it will parse from all possible node names.
    #[config(children)]
    builders: Vec<Builder>,
}

#[derive(Debug, ConfigNode, PartialEq)]
struct User {
    #[config(property)]
    passwd: String,
}

#[derive(Debug, ConfigNode, PartialEq)]
struct Config {
    #[config(child)]
    user_config: BuildSteps,

    #[config(children)]
    packages: Vec<Named<Package>>,

    #[config(children)]
    users: Vec<Named<User>>,
}

#[derive(Debug, ConfigNode, PartialEq)]
#[config(node_name("build"))] // Rename to config without this the name would have been build-steps
struct BuildSteps {
    #[config(children)]
    entries: Vec<Named<BuildStep>>,
}

#[derive(Debug, ConfigNode, PartialEq)]
enum BuildStep {
    Install {
        #[config(argument)]
        package: Box<str>,
    },

    #[config(node_name("run"))]
    RunBuilder {
        #[config(child)]
        builder: Builder,
    },

    #[config(node_name(any))]
    Unknown {
        #[config(node_name)]
        step_name: String,
    },
}

#[derive(Debug, ConfigNode, PartialEq)]
#[config(node_name(any))] // This type will parse from any input node name.
struct Builder {
    #[config(node_name)]
    name: String,

    #[config(property("out"))] // Rename a property
    output: PathBuf,
}

fn main() {
    let source_code = r#"

build {
    install "docker" name="install docker"
    install "cargo" name="install cargo"
    run name="run the builder" {
        cargo-builder out="/test"
    }

    non-existant-step name="I don't exist"
}

package name="starryconfig" {
        author "me"
}

user name="jef" passwd="pass"

package name="star" {
        author "also me"
        cargo-builder out="/usr/bin/star"
}

user name="other jef" passwd="pass"

"#;

    let config: Config = starryconfig::from_str(source_code).unwrap_or_else(|e| {
        panic!(
            "{:?}",
            Report::from(e).with_source_code(source_code.to_string())
        )
    });

    assert_eq!(
        config,
        Config {
            user_config: BuildSteps {
                entries: vec![
                    Named {
                        name: "install docker".into(),
                        inner: BuildStep::Install {
                            package: "docker".into()
                        }
                    },
                    Named {
                        name: "install cargo".into(),
                        inner: BuildStep::Install {
                            package: "cargo".into()
                        },
                    },
                    Named {
                        name: "run the builder".into(),
                        inner: BuildStep::RunBuilder {
                            builder: Builder {
                                name: "cargo-builder".into(),
                                output: "/test".into()
                            }
                        }
                    },
                    Named {
                        name: "I don't exist".into(),
                        inner: BuildStep::Unknown {
                            step_name: "non-existant-step".into()
                        }
                    }
                ]
            },
            packages: vec![
                Named {
                    name: "starryconfig".into(),
                    inner: Package {
                        author: Author { value: "me".into() },
                        builders: vec![],
                    }
                },
                Named {
                    name: "star".into(),
                    inner: Package {
                        author: Author {
                            value: "also me".into()
                        },
                        builders: vec![Builder {
                            name: "cargo-builder".into(),
                            output: "/usr/bin/star".into(),
                        }],
                    }
                }
            ],
            users: vec![
                Named {
                    name: "jef".into(),
                    inner: User {
                        passwd: "pass".into(),
                    }
                },
                Named {
                    name: "other jef".into(),
                    inner: User {
                        passwd: "pass".into(),
                    }
                },
            ]
        }
    );
}
