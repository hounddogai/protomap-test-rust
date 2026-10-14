use super::*;

/// Puts a pending allowlist forecast code in its canonical PAF- form.
pub fn normalize_pending_allowlist_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PAF-") {
        return clean.clone();
    }
    format!("PAF-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 7408.0, in order.
pub fn pending_allowlist_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7408.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
