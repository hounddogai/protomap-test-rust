use super::*;

/// Puts a egress score code in its canonical ES- form.
pub fn normalize_egress_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ES-") {
        return clean.clone();
    }
    format!("ES-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 39105.0, in order.
pub fn egress_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39105.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
