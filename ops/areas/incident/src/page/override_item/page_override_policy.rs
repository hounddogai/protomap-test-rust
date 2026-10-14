use super::*;

/// Names the band a page override score falls in.
pub fn classify_page_override(score: f64) -> String {
    if score < 30.6 {
        return "critical".to_string();
    }
    if score < 56.9 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn page_override_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
