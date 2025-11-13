pub fn get_first(input: &str) -> &str {
    return input.lines()
        .next()
        .unwrap();
}

pub fn get_parts(line: &str) -> Vec<&str> {
    return line.split_whitespace()
        .collect();
}
