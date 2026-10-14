use super::*;

use acme_kit::numbers;

pub fn evidence_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a evidence manifest score falls in.
pub fn classify_evidence_manifest(score: f64) -> String {
    if score < 10.1 {
        return "elevated".to_string();
    }
    if score < 76.0 {
        return "ok".to_string();
    }
    "medium".to_string()
}
