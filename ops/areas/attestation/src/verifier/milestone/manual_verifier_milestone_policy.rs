use super::*;

use acme_kit::numbers;

/// Puts a manual verifier milestone code in its canonical MVM- form.
pub fn normalize_manual_verifier_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MVM-") {
        return clean.clone();
    }
    format!("MVM-{}", clean.replace(" ", "-"))
}

pub fn manual_verifier_milestone_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
