pub fn get_all(input: &str) -> Vec<&str> {
    return input.lines()
        .collect();
}

pub fn get_parts(line: &str) -> Vec<&str> {
    return line.split_whitespace()
        .collect();
}
