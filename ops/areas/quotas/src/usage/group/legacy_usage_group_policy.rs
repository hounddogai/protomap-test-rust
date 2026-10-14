use super::*;

/// Names the band a legacy usage group score falls in.
pub fn classify_legacy_usage_group(score: f64) -> String {
    if score < 30.2 {
        return "watch".to_string();
    }
    if score < 76.3 {
        return "elevated".to_string();
    }
    if score < 80.4 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn legacy_usage_group_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
