use super::*;

use acme_kit::numbers;

pub fn batched_escalation_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a batched escalation reservation code in its canonical BER- form.
pub fn normalize_batched_escalation_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BER-") {
        return clean.clone();
    }
    format!("BER-{}", clean.replace(" ", "-"))
}

/// Names the band a batched escalation reservation score falls in.
pub fn classify_batched_escalation_reservation(score: f64) -> String {
    if score < 2.9 {
        return "watch".to_string();
    }
    if score < 48.7 {
        return "elevated".to_string();
    }
    if score < 82.7 {
        return "medium".to_string();
    }
    if score < 84.8 {
        return "high".to_string();
    }
    "minimal".to_string()
}
