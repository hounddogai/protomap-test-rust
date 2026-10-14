use super::*;

use acme_kit::numbers;

/// Names the band a escalation score score falls in.
pub fn classify_escalation_score(score: f64) -> String {
    if score < 4.0 {
        return "medium".to_string();
    }
    if score < 4.4 {
        return "minimal".to_string();
    }
    if score < 32.6 {
        return "low".to_string();
    }
    if score < 41.6 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn escalation_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.0;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
