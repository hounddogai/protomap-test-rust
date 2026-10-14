use super::*;

pub fn scheduled_credential_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a scheduled credential segment score falls in.
pub fn classify_scheduled_credential_segment(score: f64) -> String {
    if score < 56.5 {
        return "low".to_string();
    }
    if score < 76.1 {
        return "ok".to_string();
    }
    if score < 81.1 {
        return "high".to_string();
    }
    "minimal".to_string()
}
