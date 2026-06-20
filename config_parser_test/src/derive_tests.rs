use config_parser::{ConfigNode, from_str};

#[derive(ConfigNode, Debug, PartialEq, Eq)]
struct Child {
    #[config(property, rename = "renamed")]
    i_should_be_renamed: String,
}

#[derive(ConfigNode, Debug, PartialEq, Eq)]
struct Test {
    #[config(children)]
    children: Vec<Child>,
}

#[test]
fn renamed_property() {
    let code = r#"
    test renamed="hello"
"#;
    assert_eq!(
        from_str::<Test>(code).unwrap(),
        Test {
            children: vec![Child {
                i_should_be_renamed: "hello".to_string()
            }]
        }
    );
}

#[test]
fn missing_renamed_property() {
    let code = r#"
    test
"#;
    assert_eq!(
        from_str::<Test>(code).map_err(|e| e.to_string()),
        Err("Missing property: renamed.".to_string())
    );
}

#[derive(ConfigNode, Debug, PartialEq)]
struct Node1;
#[derive(ConfigNode, Debug, PartialEq)]
struct Node2;

#[derive(ConfigNode, Debug, PartialEq)]
struct Test2 {
    #[config(children)]
    node1: Vec<Node1>,
    #[config(children)]
    node2: Vec<Node2>,
}

#[test]
fn multiple_config_children_in_one_node() {
    let code = r#"
node1
node1
node2
node1
"#;

    assert_eq!(
        from_str::<Test2>(code).unwrap(),
        Test2 {
            node1: vec![Node1, Node1, Node1],
            node2: vec![Node2]
        }
    );
}

#[derive(ConfigNode, Debug, PartialEq)]
struct Var1 {
    #[config(property)]
    var1_prop: bool,
}
#[derive(ConfigNode, Debug, PartialEq)]
struct Var2 {
    #[config(property)]
    var2_prop: bool,
}

#[derive(ConfigNode, Debug, PartialEq)]
enum MyEnum {
    Var1(Var1),
    Var2(Var2),
}

#[derive(ConfigNode, Debug, PartialEq)]
struct Test3 {
    #[config(child)]
    enumm: MyEnum,
}

#[test]
fn enum_as_a_normal_field() {
    let code = r#"
    enumm
"#;

    assert_eq!(
        from_str::<Test3>(code).unwrap(),
        Test3 {
            enumm: MyEnum::Var1(Var1 { var1_prop: true })
        }
    );
}
