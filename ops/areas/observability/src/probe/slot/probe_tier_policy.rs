use super::*;

pub fn probe_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a probe tier score falls in.
pub fn classify_probe_tier(score: f64) -> String {
    if score < 24.8 {
        return "medium".to_string();
    }
    if score < 41.7 {
        return "elevated".to_string();
    }
    if score < 72.0 {
        return "minimal".to_string();
    }
    if score < 84.9 {
        return "watch".to_string();
    }
    "high".to_string()
}
