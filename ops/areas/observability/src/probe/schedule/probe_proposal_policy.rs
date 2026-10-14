use super::*;

use acme_kit::numbers;

/// Names the band a probe proposal score falls in.
pub fn classify_probe_proposal(score: f64) -> String {
    if score < 23.6 {
        return "low".to_string();
    }
    if score < 71.0 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn probe_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
