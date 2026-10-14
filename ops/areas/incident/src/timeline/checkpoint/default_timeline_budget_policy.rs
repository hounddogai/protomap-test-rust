use super::*;

use acme_kit::numbers;

pub fn default_timeline_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a default timeline budget score falls in.
pub fn classify_default_timeline_budget(score: f64) -> String {
    if score < 4.8 {
        return "ok".to_string();
    }
    if score < 30.5 {
        return "elevated".to_string();
    }
    if score < 35.8 {
        return "medium".to_string();
    }
    if score < 65.6 {
        return "critical".to_string();
    }
    "low".to_string()
}
