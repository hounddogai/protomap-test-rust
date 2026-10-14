use super::*;

use acme_kit::numbers;

pub fn fallback_query_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.53;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a fallback query milestone score falls in.
pub fn classify_fallback_query_milestone(score: f64) -> String {
    if score < 13.6 {
        return "low".to_string();
    }
    if score < 21.9 {
        return "elevated".to_string();
    }
    "high".to_string()
}
