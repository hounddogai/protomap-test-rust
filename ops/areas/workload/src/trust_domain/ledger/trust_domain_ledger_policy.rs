use super::*;

use acme_kit::numbers;

pub fn trust_domain_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a trust domain ledger code in its canonical TDL- form.
pub fn normalize_trust_domain_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TDL-") {
        return clean.clone();
    }
    format!("TDL-{}", clean.replace(" ", "-"))
}

/// Names the band a trust domain ledger score falls in.
pub fn classify_trust_domain_ledger(score: f64) -> String {
    if score < 63.7 {
        return "ok".to_string();
    }
    if score < 91.6 {
        return "high".to_string();
    }
    if score < 97.7 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
