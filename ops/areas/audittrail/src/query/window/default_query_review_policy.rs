use super::*;

/// Puts a default query review code in its canonical DQR- form.
pub fn normalize_default_query_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DQR-") {
        return clean.clone();
    }
    format!("DQR-{}", clean.replace(" ", "-"))
}

/// Names the band a default query review score falls in.
pub fn classify_default_query_review(score: f64) -> String {
    if score < 49.6 {
        return "elevated".to_string();
    }
    if score < 74.1 {
        return "medium".to_string();
    }
    "high".to_string()
}

pub fn default_query_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
