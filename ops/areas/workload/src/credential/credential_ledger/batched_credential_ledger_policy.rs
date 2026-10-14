use super::*;

use acme_kit::numbers;

/// Puts a batched credential ledger code in its canonical BCL- form.
pub fn normalize_batched_credential_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BCL-") {
        return clean.clone();
    }
    format!("BCL-{}", clean.replace(" ", "-"))
}

pub fn batched_credential_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
