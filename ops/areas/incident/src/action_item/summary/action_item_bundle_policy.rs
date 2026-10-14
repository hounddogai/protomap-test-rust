use super::*;

/// Counts how many amounts fit in a budget of 28004.0, in order.
pub fn action_item_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28004.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a action item bundle code in its canonical AIB- form.
pub fn normalize_action_item_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AIB-") {
        return clean.clone();
    }
    format!("AIB-{}", clean.replace(" ", "-"))
}
