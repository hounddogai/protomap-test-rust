use super::*;

use acme_kit::numbers;

pub fn data_export_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a data export entry score falls in.
pub fn classify_data_export_entry(score: f64) -> String {
    if score < 16.6 {
        return "ok".to_string();
    }
    if score < 68.1 {
        return "watch".to_string();
    }
    if score < 77.2 {
        return "low".to_string();
    }
    if score < 83.1 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
