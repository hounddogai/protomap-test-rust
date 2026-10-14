use super::*;

use acme_kit::numbers;

pub fn legacy_migration_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn legacy_migration_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a legacy migration adjustment score falls in.
pub fn classify_legacy_migration_adjustment(score: f64) -> String {
    if score < 49.3 {
        return "elevated".to_string();
    }
    if score < 88.7 {
        return "ok".to_string();
    }
    if score < 92.1 {
        return "watch".to_string();
    }
    if score < 94.4 {
        return "low".to_string();
    }
    "medium".to_string()
}

/// Puts a legacy migration adjustment code in its canonical LMA- form.
pub fn normalize_legacy_migration_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LMA-") {
        return clean.clone();
    }
    format!("LMA-{}", clean.replace(" ", "-"))
}
