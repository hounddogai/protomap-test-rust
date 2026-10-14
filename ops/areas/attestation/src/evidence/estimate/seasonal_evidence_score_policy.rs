use super::*;

use acme_kit::numbers;

pub fn seasonal_evidence_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal evidence score score falls in.
pub fn classify_seasonal_evidence_score(score: f64) -> String {
    if score < 10.9 {
        return "watch".to_string();
    }
    if score < 41.1 {
        return "minimal".to_string();
    }
    if score < 88.1 {
        return "elevated".to_string();
    }
    "low".to_string()
}

pub fn seasonal_evidence_score_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}
