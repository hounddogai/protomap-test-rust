use super::*;

use acme_kit::numbers;

/// Puts a batched policy baseline code in its canonical BPB- form.
pub fn normalize_batched_policy_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BPB-") {
        return clean.clone();
    }
    format!("BPB-{}", clean.replace(" ", "-"))
}

pub fn batched_policy_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
