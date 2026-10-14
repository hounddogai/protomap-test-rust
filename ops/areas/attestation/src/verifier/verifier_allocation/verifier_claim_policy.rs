use super::*;

use acme_kit::numbers;

/// Names the band a verifier claim score falls in.
pub fn classify_verifier_claim(score: f64) -> String {
    if score < 4.1 {
        return "low".to_string();
    }
    if score < 84.5 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn verifier_claim_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
