use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 89853.0, in order.
pub fn postmortem_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89853.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a postmortem budget score falls in.
pub fn classify_postmortem_budget(score: f64) -> String {
    if score < 35.1 {
        return "high".to_string();
    }
    if score < 61.9 {
        return "low".to_string();
    }
    if score < 84.0 {
        return "medium".to_string();
    }
    if score < 88.0 {
        return "ok".to_string();
    }
    "elevated".to_string()
}

/// Puts a postmortem budget code in its canonical PB- form.
pub fn normalize_postmortem_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PB-") {
        return clean.clone();
    }
    format!("PB-{}", clean.replace(" ", "-"))
}

pub fn postmortem_budget_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
