use super::*;

use acme_kit::numbers;

/// Names the band a capped grant ledger score falls in.
pub fn classify_capped_grant_ledger(score: f64) -> String {
    if score < 1.4 {
        return "minimal".to_string();
    }
    if score < 2.6 {
        return "high".to_string();
    }
    if score < 57.1 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn capped_grant_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a capped grant ledger code in its canonical CGL- form.
pub fn normalize_capped_grant_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CGL-") {
        return clean.clone();
    }
    format!("CGL-{}", clean.replace(" ", "-"))
}
