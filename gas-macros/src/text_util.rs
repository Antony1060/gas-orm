pub(crate) fn pascal_to_snake_case(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + src.len() / 2);

    for (idx, char) in src.chars().enumerate() {
        if idx != 0 && char.is_ascii_uppercase() {
            out.push('_');
        }

        out.push(char.to_ascii_lowercase());
    }

    out
}
