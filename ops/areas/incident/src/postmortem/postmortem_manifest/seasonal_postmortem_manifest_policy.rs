use super::*;

/// Names the band a seasonal postmortem manifest score falls in.
pub fn classify_seasonal_postmortem_manifest(score: f64) -> String {
    if score < 26.4 {
        return "low".to_string();
    }
    if score < 26.6 {
        return "medium".to_string();
    }
    if score < 59.2 {
        return "high".to_string();
    }
    if score < 79.2 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn seasonal_postmortem_manifest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
