use super::*;

use acme_kit::numbers;

/// Names the band a page snapshot score falls in.
pub fn classify_page_snapshot(score: f64) -> String {
    if score < 21.8 {
        return "medium".to_string();
    }
    if score < 26.0 {
        return "minimal".to_string();
    }
    if score < 44.8 {
        return "ok".to_string();
    }
    if score < 94.3 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn page_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
