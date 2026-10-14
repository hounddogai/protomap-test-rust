use super::*;

use acme_kit::numbers;

pub fn strict_verifier_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a strict verifier rollup score falls in.
pub fn classify_strict_verifier_rollup(score: f64) -> String {
    if score < 59.4 {
        return "critical".to_string();
    }
    if score < 82.5 {
        return "high".to_string();
    }
    if score < 83.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Puts a strict verifier rollup code in its canonical SVR- form.
pub fn normalize_strict_verifier_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SVR-") {
        return clean.clone();
    }
    format!("SVR-{}", clean.replace(" ", "-"))
}
