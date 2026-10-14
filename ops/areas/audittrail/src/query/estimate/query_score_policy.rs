use super::*;

use acme_kit::numbers;

pub fn query_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

pub fn query_score_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a query score score falls in.
pub fn classify_query_score(score: f64) -> String {
    if score < 20.6 {
        return "low".to_string();
    }
    if score < 86.1 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a query score code in its canonical QS- form.
pub fn normalize_query_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QS-") {
        return clean.clone();
    }
    format!("QS-{}", clean.replace(" ", "-"))
}
