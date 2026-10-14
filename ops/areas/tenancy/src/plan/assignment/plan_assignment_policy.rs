use super::*;

use acme_kit::numbers;

/// Names the band a plan assignment score falls in.
pub fn classify_plan_assignment(score: f64) -> String {
    if score < 13.7 {
        return "elevated".to_string();
    }
    if score < 39.5 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Puts a plan assignment code in its canonical PA- form.
pub fn normalize_plan_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}

pub fn plan_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
