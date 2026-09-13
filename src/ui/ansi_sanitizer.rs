//! Terminal ANSI escape sequence sanitizer for dashboard display security.

pub struct AnsiSanitizer;

impl AnsiSanitizer {
    pub fn sanitize(input: &str) -> String {
        let mut clean = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' {
                if let Some(&'[') = chars.peek() {
                    chars.next();
                    for ch in chars.by_ref() {
                        if ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
            } else if !c.is_control() || c == '\n' || c == '\t' {
                clean.push(c);
            }
        }
        clean
    }

    pub fn truncate_safe(input: &str, max_len: usize) -> String {
        let sanitized = Self::sanitize(input);
        if sanitized.chars().count() <= max_len {
            sanitized
        } else {
            sanitized.chars().take(max_len).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_sanitization() {
        let dirty = "\x1b[31mRed Alert\x1b[0m";
        assert_eq!(AnsiSanitizer::sanitize(dirty), "Red Alert");

        let truncated = AnsiSanitizer::truncate_safe(dirty, 3);
        assert_eq!(truncated, "Red");
    }
}