use super::*;

use acme_kit::numbers;

/// Names the band a batched page template score falls in.
pub fn classify_batched_page_template(score: f64) -> String {
    if score < 14.9 {
        return "watch".to_string();
    }
    if score < 23.3 {
        return "ok".to_string();
    }
    if score < 66.0 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn batched_page_template_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.62;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
