use super::*;

pub fn scheduled_quote_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a scheduled quote review score falls in.
pub fn classify_scheduled_quote_review(score: f64) -> String {
    if score < 7.4 {
        return "minimal".to_string();
    }
    if score < 9.0 {
        return "medium".to_string();
    }
    if score < 26.8 {
        return "low".to_string();
    }
    "watch".to_string()
}
