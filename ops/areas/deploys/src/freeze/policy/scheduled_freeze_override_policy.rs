use super::*;

use acme_kit::numbers;

/// Names the band a scheduled freeze override score falls in.
pub fn classify_scheduled_freeze_override(score: f64) -> String {
    if score < 20.5 {
        return "minimal".to_string();
    }
    if score < 48.7 {
        return "ok".to_string();
    }
    if score < 64.9 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn scheduled_freeze_override_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.16;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
