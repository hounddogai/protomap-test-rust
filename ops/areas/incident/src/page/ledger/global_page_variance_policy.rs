use super::*;

use acme_kit::numbers;

/// Names the band a global page variance score falls in.
pub fn classify_global_page_variance(score: f64) -> String {
    if score < 6.3 {
        return "minimal".to_string();
    }
    if score < 60.4 {
        return "critical".to_string();
    }
    "watch".to_string()
}

/// Puts a global page variance code in its canonical GPV- form.
pub fn normalize_global_page_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GPV-") {
        return clean.clone();
    }
    format!("GPV-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 60726.0, in order.
pub fn global_page_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60726.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_page_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
