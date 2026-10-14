use super::*;

use acme_kit::numbers;

pub fn seasonal_rollout_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.45;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal rollout proposal score falls in.
pub fn classify_seasonal_rollout_proposal(score: f64) -> String {
    if score < 1.2 {
        return "critical".to_string();
    }
    if score < 3.7 {
        return "minimal".to_string();
    }
    if score < 4.7 {
        return "high".to_string();
    }
    "low".to_string()
}
