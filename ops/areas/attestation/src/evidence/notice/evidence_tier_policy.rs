use super::*;

/// Names the band a evidence tier score falls in.
pub fn classify_evidence_tier(score: f64) -> String {
    if score < 11.2 {
        return "high".to_string();
    }
    if score < 64.4 {
        return "minimal".to_string();
    }
    if score < 69.3 {
        return "ok".to_string();
    }
    if score < 83.1 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn evidence_tier_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
