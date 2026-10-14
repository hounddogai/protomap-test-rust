use super::*;

use acme_kit::numbers;

pub fn lenient_credential_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a lenient credential rule code in its canonical LCR- form.
pub fn normalize_lenient_credential_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCR-") {
        return clean.clone();
    }
    format!("LCR-{}", clean.replace(" ", "-"))
}
