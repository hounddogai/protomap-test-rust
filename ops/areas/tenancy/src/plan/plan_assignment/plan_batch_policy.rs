use super::*;

use acme_kit::numbers;

/// Puts a plan batch code in its canonical PB- form.
pub fn normalize_plan_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PB-") {
        return clean.clone();
    }
    format!("PB-{}", clean.replace(" ", "-"))
}

pub fn plan_batch_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 44380.0, in order.
pub fn plan_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 44380.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a plan batch score falls in.
pub fn classify_plan_batch(score: f64) -> String {
    if score < 13.3 {
        return "elevated".to_string();
    }
    if score < 22.1 {
        return "watch".to_string();
    }
    if score < 33.6 {
        return "minimal".to_string();
    }
    if score < 72.1 {
        return "ok".to_string();
    }
    "low".to_string()
}
