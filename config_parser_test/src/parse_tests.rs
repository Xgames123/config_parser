use config_parser::{ConfigNode, ConfigValue, Document};
use miette::Report;

fn parse<'c>(code: &'c str) -> Document<'c> {
    match Document::from_str(code) {
        Ok(v) => v,
        Err(e) => {
            panic!("{:?}", Report::new(e).with_source_code(code.to_string()))
        }
    }
}

fn test(code: &str, expected: Document) {
    let parsed = parse(code);
    let parsed_node = parsed.into_node();
    let expected_node = expected.into_node();

    if !parsed_node.eq_no_span(&expected_node) {
        assert!(false, "{:?}\nParsed does not match expected", parsed_node)
    }
}

fn test_error(code: &str, expected_message: &str) {
    match Document::from_str(&code) {
        Ok(_) => panic!("test_error succeeded but should throw an error"),
        Err(e) => {
            let message = match &e {
                config_parser::ConfigError::Syntax { inner } => inner.to_string(),
                e => e.to_string(),
            };

            let report = Report::new(e).with_source_code(code.to_string());
            println!("{:?}", report);

            assert_eq!(message, expected_message, "Error message doesn't match");
        }
    }
}

#[test]
fn test1() {
    test(
        "
test {

}
",
        Document::new([ConfigNode::new("test")]),
    )
}

#[test]
fn boolean_test() {
    test(
        "
test true #true false #false my_prop=#true my_prop2=false
",
        Document::new([ConfigNode::new("test")
            .with_arg(ConfigValue::Bool(true))
            .with_arg(ConfigValue::Bool(true))
            .with_arg(ConfigValue::Bool(false))
            .with_prop("my_prop", ConfigValue::Bool(true))
            .with_prop("my_prop2", ConfigValue::Bool(false))]),
    );
}

#[test]
fn property_test() {
    test(
        "\nchild property=\"string\"\n",
        Document::new([
            ConfigNode::new("child").with_prop("property", ConfigValue::String("string"))
        ]),
    );
}

#[test]
fn same_line_node_children() {
    test(
        "
my_node { child_node }
",
        Document::new([ConfigNode::new("my_node").with_child(ConfigNode::new("child_node"))]),
    );
}

#[test]
fn curlies_in_string() {
    test(
        r#"
  content {
    // copy "LICENCE.md" dest="/usr/share/licenses/diststar/" if="target.unix"
    // copy "LICENCE.md" dest="%ProgramFiles%/diststar/" if="!target.unix"
    cargo bin="starpack" dest="${binary_path}"
  }
"#,
        Document::new([ConfigNode::new("content").with_child(
            ConfigNode::new("cargo")
                .with_prop("bin", ConfigValue::String("starpack"))
                .with_prop("dest", ConfigValue::String("${binary_path}")),
        )]),
    )
}

#[test]
fn parse_node_multiple_of_line_comments() {
    test(
        "test_node {
//fake_child
//fake_child \"2\"
real_child
}",
        Document::new([ConfigNode::new("test_node").with_child(ConfigNode::new("real_child"))]),
    );
}

#[test]
fn invalid_prop_arg_order() {
    test_error(
        "test_node {
my_node my_prop=attr \"my_arg\"
}",
        "Expected a property but got an argument",
    );
}

#[test]
fn invalid_argument() {
    test_error(
        "test_node {
my_node \"next is an invalid attribute\" $attr
}",
        "Ident strings can only contain the characters: a-z_-.",
    );
}

#[test]
fn invalid_property() {
    test_error(
        "test_node {
my_node \"next is an invalid property\" prop=$attr
}",
        "Ident strings can only contain the characters: a-z_-.",
    );
}
