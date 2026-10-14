use super::*;

use acme_kit::numbers;

/// Names the band a action item request score falls in.
pub fn classify_action_item_request(score: f64) -> String {
    if score < 14.0 {
        return "elevated".to_string();
    }
    if score < 32.8 {
        return "medium".to_string();
    }
    if score < 75.3 {
        return "critical".to_string();
    }
    if score < 97.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn action_item_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
