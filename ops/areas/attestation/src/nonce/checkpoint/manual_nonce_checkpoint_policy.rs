use super::*;

/// Puts a manual nonce checkpoint code in its canonical MNC- form.
pub fn normalize_manual_nonce_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MNC-") {
        return clean.clone();
    }
    format!("MNC-{}", clean.replace(" ", "-"))
}

pub fn manual_nonce_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
