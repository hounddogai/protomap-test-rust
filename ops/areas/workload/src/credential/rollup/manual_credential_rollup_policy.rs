use super::*;

use acme_kit::numbers;

pub fn manual_credential_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a manual credential rollup score falls in.
pub fn classify_manual_credential_rollup(score: f64) -> String {
    if score < 52.4 {
        return "critical".to_string();
    }
    if score < 85.4 {
        return "ok".to_string();
    }
    if score < 96.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}
