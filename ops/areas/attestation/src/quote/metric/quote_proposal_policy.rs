use super::*;

use acme_kit::numbers;

/// Puts a quote proposal code in its canonical QP- form.
pub fn normalize_quote_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("QP-") {
        return clean.clone();
    }
    format!("QP-{}", clean.replace(" ", "-"))
}

pub fn quote_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
