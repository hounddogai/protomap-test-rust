use super::*;

/// Names the band a batched query tier score falls in.
pub fn classify_batched_query_tier(score: f64) -> String {
    if score < 3.9 {
        return "critical".to_string();
    }
    if score < 19.8 {
        return "minimal".to_string();
    }
    if score < 51.2 {
        return "low".to_string();
    }
    if score < 84.0 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Puts a batched query tier code in its canonical BQT- form.
pub fn normalize_batched_query_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BQT-") {
        return clean.clone();
    }
    format!("BQT-{}", clean.replace(" ", "-"))
}

pub fn batched_query_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
