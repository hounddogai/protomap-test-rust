use super::*;

use acme_kit::numbers;

/// Names the band a global peering group score falls in.
pub fn classify_global_peering_group(score: f64) -> String {
    if score < 26.6 {
        return "elevated".to_string();
    }
    if score < 61.0 {
        return "minimal".to_string();
    }
    if score < 73.1 {
        return "high".to_string();
    }
    if score < 94.3 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn global_peering_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
