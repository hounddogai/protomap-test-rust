use super::*;

use acme_kit::numbers;

pub fn default_migration_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Names the band a default migration variance score falls in.
pub fn classify_default_migration_variance(score: f64) -> String {
    if score < 26.2 {
        return "high".to_string();
    }
    if score < 68.8 {
        return "ok".to_string();
    }
    if score < 75.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn default_migration_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
