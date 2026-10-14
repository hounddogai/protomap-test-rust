use super::*;

use acme_kit::numbers;

pub fn capped_runbook_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a capped runbook request score falls in.
pub fn classify_capped_runbook_request(score: f64) -> String {
    if score < 49.0 {
        return "medium".to_string();
    }
    if score < 55.8 {
        return "critical".to_string();
    }
    if score < 70.4 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a capped runbook request code in its canonical CRR- form.
pub fn normalize_capped_runbook_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CRR-") {
        return clean.clone();
    }
    format!("CRR-{}", clean.replace(" ", "-"))
}
