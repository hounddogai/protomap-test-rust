use super::*;

/// Puts a legacy identity ledger code in its canonical LIL- form.
pub fn normalize_legacy_identity_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LIL-") {
        return clean.clone();
    }
    format!("LIL-{}", clean.replace(" ", "-"))
}

/// Names the band a legacy identity ledger score falls in.
pub fn classify_legacy_identity_ledger(score: f64) -> String {
    if score < 35.2 {
        return "ok".to_string();
    }
    if score < 98.2 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
