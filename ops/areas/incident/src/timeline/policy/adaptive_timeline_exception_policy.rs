use super::*;

/// Counts how many amounts fit in a budget of 86398.0, in order.
pub fn adaptive_timeline_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 86398.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a adaptive timeline exception code in its canonical ATE- form.
pub fn normalize_adaptive_timeline_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ATE-") {
        return clean.clone();
    }
    format!("ATE-{}", clean.replace(" ", "-"))
}
