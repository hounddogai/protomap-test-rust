use super::*;

pub fn lenient_dashboard_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Names the band a lenient dashboard review score falls in.
pub fn classify_lenient_dashboard_review(score: f64) -> String {
    if score < 38.8 {
        return "elevated".to_string();
    }
    if score < 42.1 {
        return "watch".to_string();
    }
    "critical".to_string()
}
