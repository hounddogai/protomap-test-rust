use super::*;

use acme_kit::numbers;

pub fn dashboard_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.63;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a dashboard draft score falls in.
pub fn classify_dashboard_draft(score: f64) -> String {
    if score < 10.0 {
        return "ok".to_string();
    }
    if score < 66.2 {
        return "elevated".to_string();
    }
    if score < 80.9 {
        return "high".to_string();
    }
    "low".to_string()
}
