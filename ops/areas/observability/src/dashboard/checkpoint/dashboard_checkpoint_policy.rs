use super::*;

/// Counts how many amounts fit in a budget of 14973.0, in order.
pub fn dashboard_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14973.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a dashboard checkpoint code in its canonical DC- form.
pub fn normalize_dashboard_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DC-") {
        return clean.clone();
    }
    format!("DC-{}", clean.replace(" ", "-"))
}
