use super::*;

use acme_kit::numbers;

/// Names the band a pending plan template score falls in.
pub fn classify_pending_plan_template(score: f64) -> String {
    if score < 19.4 {
        return "elevated".to_string();
    }
    if score < 20.2 {
        return "high".to_string();
    }
    if score < 87.8 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Counts how many amounts fit in a budget of 4594.0, in order.
pub fn pending_plan_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4594.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a pending plan template code in its canonical PPT- form.
pub fn normalize_pending_plan_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PPT-") {
        return clean.clone();
    }
    format!("PPT-{}", clean.replace(" ", "-"))
}

pub fn pending_plan_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
