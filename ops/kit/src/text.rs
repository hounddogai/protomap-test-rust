//! Small string helpers shared by every area.

/// Trims a value and lowers its case, so lookups ignore how people typed it.
pub fn normalize(s: &str) -> String {
    let trimmed = s.trim().to_string();
    if (trimmed.len() as i64) == 0 {
        return "".to_string();
    }
    trimmed.to_lowercase()
}

/// Turns a display name into a lowercase, dash-separated key.
pub fn slugify(s: &str) -> String {
    let parts = s
        .trim()
        .to_lowercase()
        .split(" ")
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut kept: Vec<String> = Vec::new();
    for part in parts.iter() {
        if (part.len() as i64) > 0 {
            kept.push(part.clone());
        }
    }
    kept.join("-")
}

/// Shortens a value to at most limit characters, marking the cut with an ellipsis.
pub fn truncate(s: &str, limit: i64) -> String {
    if (s.len() as i64) <= limit {
        return s.to_string();
    }
    if limit <= 3 {
        return s[..limit as usize].to_string();
    }
    format!("{}{}", s[..(limit - 3) as usize].to_string(), "...")
}

/// Masks all but the last keep characters, as receipts and logs show references.
pub fn mask_tail(s: &str, keep: i64) -> String {
    if (s.len() as i64) <= keep {
        return s.to_string();
    }
    let hidden = (s.len() as i64) - keep;
    format!(
        "{}{}",
        "*".repeat(hidden as usize),
        s[s.len() - keep as usize..].to_string()
    )
}

pub fn word_count(s: &str) -> i64 {
    let mut count = 0;
    for word in s.trim().split(" ").map(str::to_string).collect::<Vec<_>>() {
        if (word.len() as i64) > 0 {
            count += 1;
        }
    }
    count
}

/// Reports whether any of the words occurs in s, ignoring case.
pub fn contains_any(s: &str, words: &[String]) -> bool {
    let lowered = s.to_lowercase();
    for word in words.iter() {
        if lowered.contains(&word.to_lowercase()) {
            return true;
        }
    }
    false
}

pub fn join_non_empty(parts: &[String], sep: &str) -> String {
    let mut kept: Vec<String> = Vec::new();
    for part in parts.iter() {
        let clean = part.trim().to_string();
        if clean != "" {
            kept.push(clean);
        }
    }
    kept.join(sep)
}

pub fn first_non_empty(parts: &[String]) -> String {
    for part in parts.iter() {
        if part.trim() != "" {
            return part.clone();
        }
    }
    "".to_string()
}

/// Pads or cuts a value to exactly width characters, for fixed-width output.
pub fn fit(s: &str, width: i64) -> String {
    if (s.len() as i64) > width {
        return s[..width as usize].to_string();
    }
    format!("{}{}", s, " ".repeat((width as usize).saturating_sub(s.len())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_and_space() {
        assert_eq!(normalize("  Mixed Case "), "mixed case", "normalizes case and space");
    }

    #[test]
    fn slugifies_names() {
        assert_eq!(slugify(" Spring  Sale 2026 "), "spring-sale-2026", "slugifies names");
    }

    #[test]
    fn truncates_long_values() {
        assert_eq!(truncate("abcdefghij", 6), "abc...", "truncates long values");
    }

    #[test]
    fn keeps_short_values() {
        assert_eq!(truncate("abc", 6), "abc", "keeps short values");
    }

    #[test]
    fn masks_all_but_the_tail() {
        assert_eq!(
            mask_tail("4111111111111111", 4),
            "************1111",
            "masks all but the tail"
        );
    }

    #[test]
    fn counts_words() {
        assert_eq!(word_count("  one two  three "), 3, "counts words");
    }

    #[test]
    fn fits_to_a_width() {
        assert_eq!(fit("ab", 4), "ab  ", "fits to a width");
    }

    #[test]
    fn joins_only_non_empty_parts() {
        assert_eq!(
            join_non_empty(&vec!["a".to_string(), " ".to_string(), "b".to_string()], "/"),
            "a/b",
            "joined"
        );
        assert!(contains_any("Late Fee", &vec!["fee".to_string()]), "contains fee");
    }
}
