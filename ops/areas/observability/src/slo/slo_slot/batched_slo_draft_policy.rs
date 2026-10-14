use super::*;

/// Names the band a batched slo draft score falls in.
pub fn classify_batched_slo_draft(score: f64) -> String {
    if score < 40.6 {
        return "ok".to_string();
    }
    if score < 45.4 {
        return "watch".to_string();
    }
    if score < 72.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn batched_slo_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
