use super::*;

use acme_kit::numbers;

pub fn manual_flow_log_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a manual flow log digest code in its canonical MFLD- form.
pub fn normalize_manual_flow_log_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MFLD-") {
        return clean.clone();
    }
    format!("MFLD-{}", clean.replace(" ", "-"))
}

/// Names the band a manual flow log digest score falls in.
pub fn classify_manual_flow_log_digest(score: f64) -> String {
    if score < 28.4 {
        return "minimal".to_string();
    }
    if score < 52.0 {
        return "low".to_string();
    }
    if score < 55.4 {
        return "medium".to_string();
    }
    if score < 59.0 {
        return "critical".to_string();
    }
    "high".to_string()
}
