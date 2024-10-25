use std::path::Path;

#[cfg(not(target_os = "windows"))]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
pub fn adjust_canonicalization<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}

/// Strips the trailing newline from a string.
pub fn strip_trailing_newline(input: &String) -> String {
    input.strip_suffix("\r\n").or(input.strip_suffix("\n")).unwrap_or(input).trim().to_string()
}
