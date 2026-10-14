use super::*;

use acme_kit::numbers;

/// Puts a tiered nonce target code in its canonical TNT- form.
pub fn normalize_tiered_nonce_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TNT-") {
        return clean.clone();
    }
    format!("TNT-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered nonce target score falls in.
pub fn classify_tiered_nonce_target(score: f64) -> String {
    if score < 13.6 {
        return "elevated".to_string();
    }
    if score < 18.3 {
        return "low".to_string();
    }
    if score < 54.5 {
        return "minimal".to_string();
    }
    if score < 58.2 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn tiered_nonce_target_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
