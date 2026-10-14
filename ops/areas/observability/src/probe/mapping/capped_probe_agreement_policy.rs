use super::*;

use acme_kit::numbers;

pub fn capped_probe_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.71;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a capped probe agreement score falls in.
pub fn classify_capped_probe_agreement(score: f64) -> String {
    if score < 38.8 {
        return "high".to_string();
    }
    if score < 94.8 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

/// Puts a capped probe agreement code in its canonical CPA- form.
pub fn normalize_capped_probe_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CPA-") {
        return clean.clone();
    }
    format!("CPA-{}", clean.replace(" ", "-"))
}

pub fn capped_probe_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
