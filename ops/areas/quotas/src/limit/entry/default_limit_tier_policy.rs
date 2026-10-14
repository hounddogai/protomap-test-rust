use super::*;

/// Puts a default limit tier code in its canonical DLT- form.
pub fn normalize_default_limit_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DLT-") {
        return clean.clone();
    }
    format!("DLT-{}", clean.replace(" ", "-"))
}

/// Names the band a default limit tier score falls in.
pub fn classify_default_limit_tier(score: f64) -> String {
    if score < 6.2 {
        return "ok".to_string();
    }
    if score < 39.3 {
        return "medium".to_string();
    }
    if score < 89.9 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn default_limit_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
