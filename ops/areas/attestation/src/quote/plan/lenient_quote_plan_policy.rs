use super::*;

use acme_kit::numbers;

/// Names the band a lenient quote plan score falls in.
pub fn classify_lenient_quote_plan(score: f64) -> String {
    if score < 15.4 {
        return "medium".to_string();
    }
    if score < 31.8 {
        return "watch".to_string();
    }
    if score < 63.4 {
        return "elevated".to_string();
    }
    if score < 79.6 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Puts a lenient quote plan code in its canonical LQP- form.
pub fn normalize_lenient_quote_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LQP-") {
        return clean.clone();
    }
    format!("LQP-{}", clean.replace(" ", "-"))
}

pub fn lenient_quote_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
