use super::*;

use acme_kit::numbers;

/// Puts a egress plan code in its canonical EP- form.
pub fn normalize_egress_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EP-") {
        return clean.clone();
    }
    format!("EP-{}", clean.replace(" ", "-"))
}

pub fn egress_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
