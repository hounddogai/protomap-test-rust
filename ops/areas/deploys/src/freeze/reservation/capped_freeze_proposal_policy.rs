use super::*;

use acme_kit::numbers;

pub fn capped_freeze_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Puts a capped freeze proposal code in its canonical CFP- form.
pub fn normalize_capped_freeze_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CFP-") {
        return clean.clone();
    }
    format!("CFP-{}", clean.replace(" ", "-"))
}

/// Names the band a capped freeze proposal score falls in.
pub fn classify_capped_freeze_proposal(score: f64) -> String {
    if score < 36.8 {
        return "ok".to_string();
    }
    if score < 47.3 {
        return "watch".to_string();
    }
    if score < 68.7 {
        return "critical".to_string();
    }
    if score < 99.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}
