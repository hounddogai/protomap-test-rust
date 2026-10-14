use super::*;

use acme_kit::numbers;

/// Names the band a seasonal plan profile score falls in.
pub fn classify_seasonal_plan_profile(score: f64) -> String {
    if score < 3.4 {
        return "medium".to_string();
    }
    if score < 13.8 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn seasonal_plan_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.18;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
