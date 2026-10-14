use super::*;

use acme_kit::numbers;

pub fn postmortem_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a postmortem override score falls in.
pub fn classify_postmortem_override(score: f64) -> String {
    if score < 7.7 {
        return "low".to_string();
    }
    if score < 70.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
