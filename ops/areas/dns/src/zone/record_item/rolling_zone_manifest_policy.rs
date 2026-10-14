use super::*;

pub fn rolling_zone_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}

/// Names the band a rolling zone manifest score falls in.
pub fn classify_rolling_zone_manifest(score: f64) -> String {
    if score < 8.3 {
        return "minimal".to_string();
    }
    if score < 29.2 {
        return "watch".to_string();
    }
    "high".to_string()
}
