//! String formatting helpers.

/// Shorten a string to its first and last `qty / 2` characters joined by an
/// ellipsis — e.g. `shorten("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR", 8)`
/// → `"FRGk...WcPR"`. Strings of `qty` characters or fewer are returned
/// unchanged.
pub fn shorten(value: &str, qty: usize) -> String {
    if value.len() > qty {
        let chars_to_show = qty / 2;
        let start = &value[..chars_to_show];
        let end = &value[value.len() - chars_to_show..];
        format!("{}...{}", start, end)
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::shorten;

    #[test]
    fn test_shorten_long_string() {
        assert_eq!(
            shorten("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR", 8),
            "FRGk...WcPR"
        );
    }

    #[test]
    fn test_shorten_short_string_unchanged() {
        assert_eq!(shorten("FRGkJho6", 8), "FRGkJho6");
    }
}
