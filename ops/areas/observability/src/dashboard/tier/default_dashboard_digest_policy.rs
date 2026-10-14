use super::*;

use acme_kit::numbers;

pub fn default_dashboard_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.75;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn default_dashboard_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a default dashboard digest code in its canonical DDD- form.
pub fn normalize_default_dashboard_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDD-") {
        return clean.clone();
    }
    format!("DDD-{}", clean.replace(" ", "-"))
}

/// Names the band a default dashboard digest score falls in.
pub fn classify_default_dashboard_digest(score: f64) -> String {
    if score < 49.4 {
        return "minimal".to_string();
    }
    if score < 50.9 {
        return "elevated".to_string();
    }
    if score < 54.5 {
        return "ok".to_string();
    }
    if score < 74.6 {
        return "low".to_string();
    }
    "high".to_string()
}
