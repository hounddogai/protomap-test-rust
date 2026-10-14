use super::*;

/// Names the band a identity estimate score falls in.
pub fn classify_identity_estimate(score: f64) -> String {
    if score < 4.7 {
        return "ok".to_string();
    }
    if score < 99.2 {
        return "high".to_string();
    }
    "low".to_string()
}

pub fn identity_estimate_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
