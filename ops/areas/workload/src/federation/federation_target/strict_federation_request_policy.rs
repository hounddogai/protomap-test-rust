use super::*;

/// Counts how many amounts fit in a budget of 9491.0, in order.
pub fn strict_federation_request_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9491.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a strict federation request code in its canonical SFR- form.
pub fn normalize_strict_federation_request_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFR-") {
        return clean.clone();
    }
    format!("SFR-{}", clean.replace(" ", "-"))
}
