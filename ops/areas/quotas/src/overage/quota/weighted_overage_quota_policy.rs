use super::*;

use acme_kit::numbers;

pub fn weighted_overage_quota_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted overage quota code in its canonical WOQ- form.
pub fn normalize_weighted_overage_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WOQ-") {
        return clean.clone();
    }
    format!("WOQ-{}", clean.replace(" ", "-"))
}

/// Names the band a weighted overage quota score falls in.
pub fn classify_weighted_overage_quota(score: f64) -> String {
    if score < 2.3 {
        return "minimal".to_string();
    }
    if score < 3.0 {
        return "ok".to_string();
    }
    if score < 42.8 {
        return "elevated".to_string();
    }
    if score < 57.9 {
        return "low".to_string();
    }
    "medium".to_string()
}
