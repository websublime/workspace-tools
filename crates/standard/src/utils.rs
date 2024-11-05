/// Strips the trailing newline from a string.
pub fn strip_trailing_newline(input: &String) -> String {
    input.strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(input).trim().to_string()
}
