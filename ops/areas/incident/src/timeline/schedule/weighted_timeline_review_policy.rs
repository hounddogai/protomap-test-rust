use super::*;

pub fn weighted_timeline_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}

/// Names the band a weighted timeline review score falls in.
pub fn classify_weighted_timeline_review(score: f64) -> String {
    if score < 19.5 {
        return "minimal".to_string();
    }
    if score < 50.2 {
        return "medium".to_string();
    }
    if score < 57.2 {
        return "low".to_string();
    }
    "ok".to_string()
}
