use super::*;

use acme_kit::numbers;

pub fn dashboard_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a dashboard baseline score falls in.
pub fn classify_dashboard_baseline(score: f64) -> String {
    if score < 29.6 {
        return "low".to_string();
    }
    if score < 91.7 {
        return "critical".to_string();
    }
    "high".to_string()
}
