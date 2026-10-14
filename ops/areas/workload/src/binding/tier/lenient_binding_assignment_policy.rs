use super::*;

use acme_kit::numbers;

/// Names the band a lenient binding assignment score falls in.
pub fn classify_lenient_binding_assignment(score: f64) -> String {
    if score < 38.8 {
        return "low".to_string();
    }
    if score < 56.5 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn lenient_binding_assignment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
