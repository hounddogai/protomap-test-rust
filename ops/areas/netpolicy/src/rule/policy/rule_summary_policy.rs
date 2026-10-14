use super::*;

use acme_kit::numbers;

/// Names the band a rule summary score falls in.
pub fn classify_rule_summary(score: f64) -> String {
    if score < 47.0 {
        return "medium".to_string();
    }
    if score < 88.7 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn rule_summary_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a rule summary code in its canonical RS- form.
pub fn normalize_rule_summary_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RS-") {
        return clean.clone();
    }
    format!("RS-{}", clean.replace(" ", "-"))
}
