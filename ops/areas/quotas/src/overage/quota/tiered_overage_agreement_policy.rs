use super::*;

use acme_kit::numbers;

/// Puts a tiered overage agreement code in its canonical TOA- form.
pub fn normalize_tiered_overage_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TOA-") {
        return clean.clone();
    }
    format!("TOA-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered overage agreement score falls in.
pub fn classify_tiered_overage_agreement(score: f64) -> String {
    if score < 26.6 {
        return "ok".to_string();
    }
    if score < 53.4 {
        return "critical".to_string();
    }
    if score < 65.4 {
        return "low".to_string();
    }
    if score < 94.8 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_overage_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn tiered_overage_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.46;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
