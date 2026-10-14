use super::*;

pub fn allowlist_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a allowlist target score falls in.
pub fn classify_allowlist_target(score: f64) -> String {
    if score < 10.4 {
        return "high".to_string();
    }
    if score < 39.2 {
        return "low".to_string();
    }
    "watch".to_string()
}
