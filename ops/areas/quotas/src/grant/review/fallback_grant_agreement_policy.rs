use super::*;

/// Counts how many amounts fit in a budget of 99610.0, in order.
pub fn fallback_grant_agreement_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 99610.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a fallback grant agreement code in its canonical FGA- form.
pub fn normalize_fallback_grant_agreement_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FGA-") {
        return clean.clone();
    }
    format!("FGA-{}", clean.replace(" ", "-"))
}
