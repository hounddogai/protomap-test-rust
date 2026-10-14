use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 30318.0, in order.
pub fn default_egress_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30318.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default egress rollup score falls in.
pub fn classify_default_egress_rollup(score: f64) -> String {
    if score < 17.6 {
        return "elevated".to_string();
    }
    if score < 56.2 {
        return "medium".to_string();
    }
    if score < 59.0 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn default_egress_rollup_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a default egress rollup code in its canonical DER- form.
pub fn normalize_default_egress_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DER-") {
        return clean.clone();
    }
    format!("DER-{}", clean.replace(" ", "-"))
}
