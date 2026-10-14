use super::*;

use acme_kit::numbers;

pub fn seasonal_federation_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a seasonal federation baseline code in its canonical SFB- form.
pub fn normalize_seasonal_federation_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFB-") {
        return clean.clone();
    }
    format!("SFB-{}", clean.replace(" ", "-"))
}

pub fn seasonal_federation_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.58;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal federation baseline score falls in.
pub fn classify_seasonal_federation_baseline(score: f64) -> String {
    if score < 12.2 {
        return "ok".to_string();
    }
    if score < 20.3 {
        return "high".to_string();
    }
    if score < 52.5 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
