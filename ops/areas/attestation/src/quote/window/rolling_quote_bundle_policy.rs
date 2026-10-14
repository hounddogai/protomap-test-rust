use super::*;

/// Counts how many amounts fit in a budget of 36905.0, in order.
pub fn rolling_quote_bundle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36905.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a rolling quote bundle code in its canonical RQB- form.
pub fn normalize_rolling_quote_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RQB-") {
        return clean.clone();
    }
    format!("RQB-{}", clean.replace(" ", "-"))
}
