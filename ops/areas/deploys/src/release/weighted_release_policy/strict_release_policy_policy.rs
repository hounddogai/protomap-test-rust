use super::*;

/// Puts a strict release policy code in its canonical SRP- form.
pub fn normalize_strict_release_policy_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRP-") {
        return clean.clone();
    }
    format!("SRP-{}", clean.replace(" ", "-"))
}

pub fn strict_release_policy_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a strict release policy score falls in.
pub fn classify_strict_release_policy(score: f64) -> String {
    if score < 76.0 {
        return "critical".to_string();
    }
    if score < 80.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}
