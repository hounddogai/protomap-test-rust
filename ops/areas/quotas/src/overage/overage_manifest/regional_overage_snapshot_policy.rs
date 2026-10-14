use super::*;

/// Names the band a regional overage snapshot score falls in.
pub fn classify_regional_overage_snapshot(score: f64) -> String {
    if score < 39.4 {
        return "high".to_string();
    }
    if score < 74.4 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn regional_overage_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}
