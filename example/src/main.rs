use config_parser::{ConfigNode, ConfigValue, ParseConfigNode};
use miette::{Error, Report};

mod generics;

#[derive(Debug, ConfigNode)]
struct Author {
    #[config(argument)]
    value: String,
}

#[derive(Debug, ConfigNode)]
struct Package {
    #[config(argument)]
    name: String,

    #[config(child)]
    author: Author,
}

#[derive(Debug, ConfigNode)]
struct User {
    #[config(argument)]
    name: String,
}

#[derive(Debug, ConfigNode)]
struct Config {
    #[config(child, rename = "config")]
    user_config: UserConfig,

    #[config(children)]
    packages: Vec<Package>,

    #[config(children)]
    users: Vec<User>,
}

#[derive(Debug, ConfigNode)]
struct UserConfig {
    #[config(property)]
    max_name_len: usize,
}

fn main() {
    let source_code = "

config max_name_len=100

package \"stuff\" {
        author \"me\"
}

user \"jef\"

package \"stuff 2\" {
        author \"also me\"
}

user \"jef 2\"

";

    let config: Config = config_parser::from_str(source_code).unwrap_or_else(|e| {
        panic!(
            "{:?}",
            Report::from(e).with_source_code(source_code.to_string())
        )
    });
    println!("{:?}", config);
}
