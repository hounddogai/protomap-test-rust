use super::*;

use acme_kit::numbers;

pub fn lenient_evidence_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Puts a lenient evidence plan code in its canonical LEP- form.
pub fn normalize_lenient_evidence_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LEP-") {
        return clean.clone();
    }
    format!("LEP-{}", clean.replace(" ", "-"))
}
