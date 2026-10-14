use super::*;

use acme_kit::numbers;

/// Puts a pending token digest code in its canonical PTD- form.
pub fn normalize_pending_token_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTD-") {
        return clean.clone();
    }
    format!("PTD-{}", clean.replace(" ", "-"))
}

pub fn pending_token_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.82;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
