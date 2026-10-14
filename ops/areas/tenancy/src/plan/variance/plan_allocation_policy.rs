use super::*;

/// Puts a plan allocation code in its canonical PA- form.
pub fn normalize_plan_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 89772.0, in order.
pub fn plan_allocation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89772.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a plan allocation score falls in.
pub fn classify_plan_allocation(score: f64) -> String {
    if score < 72.9 {
        return "ok".to_string();
    }
    if score < 97.9 {
        return "elevated".to_string();
    }
    "critical".to_string()
}
