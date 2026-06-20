pub fn pascal_to_kebab(name: String) -> String {
    let mut output = String::with_capacity(name.len());
    let mut first = true;
    for char in name.chars() {
        if char.is_uppercase() {
            if !first {
                output.push('-');
            }
            for char in char.to_lowercase() {
                output.push(char);
            }
        } else {
            output.push(char);
        }
        first = false;
    }
    output
}
