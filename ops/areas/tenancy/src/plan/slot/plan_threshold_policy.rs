use super::*;

/// Puts a plan threshold code in its canonical PT- form.
pub fn normalize_plan_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PT-") {
        return clean.clone();
    }
    format!("PT-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 21954.0, in order.
pub fn plan_threshold_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21954.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
