use super::*;

use acme_kit::numbers;

pub fn pending_retention_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a pending retention budget code in its canonical PRB- form.
pub fn normalize_pending_retention_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PRB-") {
        return clean.clone();
    }
    format!("PRB-{}", clean.replace(" ", "-"))
}

/// Names the band a pending retention budget score falls in.
pub fn classify_pending_retention_budget(score: f64) -> String {
    if score < 45.4 {
        return "watch".to_string();
    }
    if score < 70.3 {
        return "high".to_string();
    }
    if score < 89.9 {
        return "critical".to_string();
    }
    if score < 95.6 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
