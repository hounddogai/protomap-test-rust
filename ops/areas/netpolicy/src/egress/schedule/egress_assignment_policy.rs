use super::*;

use acme_kit::numbers;

/// Puts a egress assignment code in its canonical EA- form.
pub fn normalize_egress_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("EA-") {
        return clean.clone();
    }
    format!("EA-{}", clean.replace(" ", "-"))
}

/// Names the band a egress assignment score falls in.
pub fn classify_egress_assignment(score: f64) -> String {
    if score < 10.1 {
        return "minimal".to_string();
    }
    if score < 23.0 {
        return "elevated".to_string();
    }
    if score < 64.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}

pub fn egress_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
