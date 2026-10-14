use super::*;

use acme_kit::numbers;

pub fn evidence_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.54;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn evidence_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a evidence rollup score falls in.
pub fn classify_evidence_rollup(score: f64) -> String {
    if score < 77.6 {
        return "medium".to_string();
    }
    if score < 84.2 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
