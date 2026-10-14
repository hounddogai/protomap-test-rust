use super::*;

/// Counts how many amounts fit in a budget of 39674.0, in order.
pub fn regional_query_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39674.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a regional query profile code in its canonical RQP- form.
pub fn normalize_regional_query_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RQP-") {
        return clean.clone();
    }
    format!("RQP-{}", clean.replace(" ", "-"))
}
