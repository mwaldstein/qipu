/// Utilities for records output format
/// Escape double quotes in a string for records format.
/// Replaces `"` with `\"` to allow safe embedding in quoted fields.
pub fn escape_quotes(s: &str) -> String {
    s.replace('\"', r#"\""#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_quotes() {
        assert_eq!(escape_quotes("no quotes"), "no quotes");
        assert_eq!(escape_quotes(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(
            escape_quotes(r#"multiple "quotes" in "text""#),
            r#"multiple \"quotes\" in \"text\""#
        );
        assert_eq!(escape_quotes(""), "");
        assert_eq!(escape_quotes(r#""""#), r#"\"\""#);
    }
}
