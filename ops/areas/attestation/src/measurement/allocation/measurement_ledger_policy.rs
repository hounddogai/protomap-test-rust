use super::*;

use acme_kit::numbers;

/// Puts a measurement ledger code in its canonical ML- form.
pub fn normalize_measurement_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ML-") {
        return clean.clone();
    }
    format!("ML-{}", clean.replace(" ", "-"))
}

pub fn measurement_ledger_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.42;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a measurement ledger score falls in.
pub fn classify_measurement_ledger(score: f64) -> String {
    if score < 81.7 {
        return "watch".to_string();
    }
    if score < 87.3 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
