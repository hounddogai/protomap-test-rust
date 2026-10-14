use super::*;

pub fn batched_nonce_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a batched nonce target score falls in.
pub fn classify_batched_nonce_target(score: f64) -> String {
    if score < 21.3 {
        return "low".to_string();
    }
    if score < 26.2 {
        return "elevated".to_string();
    }
    if score < 73.4 {
        return "minimal".to_string();
    }
    if score < 90.5 {
        return "medium".to_string();
    }
    "watch".to_string()
}
