use super::*;

pub fn tiered_approval_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a tiered approval preference score falls in.
pub fn classify_tiered_approval_preference(score: f64) -> String {
    if score < 4.4 {
        return "critical".to_string();
    }
    if score < 45.9 {
        return "low".to_string();
    }
    "medium".to_string()
}
