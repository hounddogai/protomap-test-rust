use super::*;

pub fn pending_canary_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Names the band a pending canary adjustment score falls in.
pub fn classify_pending_canary_adjustment(score: f64) -> String {
    if score < 13.2 {
        return "elevated".to_string();
    }
    if score < 58.1 {
        return "medium".to_string();
    }
    if score < 71.1 {
        return "critical".to_string();
    }
    "ok".to_string()
}
