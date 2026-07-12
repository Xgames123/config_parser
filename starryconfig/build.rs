use std::fs;

fn main() {
    fs::copy("../README.md", "README.md").unwrap();
}
