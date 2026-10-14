use super::*;

/// Names the band a nonce window score falls in.
pub fn classify_nonce_window(score: f64) -> String {
    if score < 10.1 {
        return "minimal".to_string();
    }
    if score < 36.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn nonce_window_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
