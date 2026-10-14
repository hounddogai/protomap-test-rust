use super::*;

pub fn plan_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a plan preference score falls in.
pub fn classify_plan_preference(score: f64) -> String {
    if score < 32.2 {
        return "high".to_string();
    }
    if score < 44.9 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
