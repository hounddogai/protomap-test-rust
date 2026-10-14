use super::*;

/// Puts a seasonal migration score code in its canonical SMS- form.
pub fn normalize_seasonal_migration_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SMS-") {
        return clean.clone();
    }
    format!("SMS-{}", clean.replace(" ", "-"))
}

pub fn seasonal_migration_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
