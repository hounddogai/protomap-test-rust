use super::*;

use acme_kit::numbers;

/// Puts a batched federation ledger code in its canonical BFL- form.
pub fn normalize_batched_federation_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BFL-") {
        return clean.clone();
    }
    format!("BFL-{}", clean.replace(" ", "-"))
}

pub fn batched_federation_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a batched federation ledger score falls in.
pub fn classify_batched_federation_ledger(score: f64) -> String {
    if score < 59.4 {
        return "watch".to_string();
    }
    if score < 76.7 {
        return "low".to_string();
    }
    "high".to_string()
}
