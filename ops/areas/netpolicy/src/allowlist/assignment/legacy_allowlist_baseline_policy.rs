use super::*;

use acme_kit::numbers;

pub fn legacy_allowlist_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn legacy_allowlist_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a legacy allowlist baseline score falls in.
pub fn classify_legacy_allowlist_baseline(score: f64) -> String {
    if score < 24.7 {
        return "medium".to_string();
    }
    if score < 36.4 {
        return "elevated".to_string();
    }
    if score < 43.3 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
