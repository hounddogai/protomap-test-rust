use super::*;

pub fn canary_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a canary preference score falls in.
pub fn classify_canary_preference(score: f64) -> String {
    if score < 48.3 {
        return "minimal".to_string();
    }
    if score < 69.8 {
        return "watch".to_string();
    }
    if score < 70.7 {
        return "ok".to_string();
    }
    if score < 90.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}
