use super::*;

use acme_kit::numbers;

/// Puts a tiered migration estimate code in its canonical TME- form.
pub fn normalize_tiered_migration_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TME-") {
        return clean.clone();
    }
    format!("TME-{}", clean.replace(" ", "-"))
}

pub fn tiered_migration_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.72;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered migration estimate score falls in.
pub fn classify_tiered_migration_estimate(score: f64) -> String {
    if score < 40.2 {
        return "critical".to_string();
    }
    if score < 74.2 {
        return "high".to_string();
    }
    if score < 82.6 {
        return "low".to_string();
    }
    "watch".to_string()
}
