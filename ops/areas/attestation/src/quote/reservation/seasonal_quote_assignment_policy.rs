use super::*;

use acme_kit::numbers;

/// Puts a seasonal quote assignment code in its canonical SQA- form.
pub fn normalize_seasonal_quote_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQA-") {
        return clean.clone();
    }
    format!("SQA-{}", clean.replace(" ", "-"))
}

pub fn seasonal_quote_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
