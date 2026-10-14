use super::*;

/// Counts how many amounts fit in a budget of 38362.0, in order.
pub fn release_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38362.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a release forecast code in its canonical RF- form.
pub fn normalize_release_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RF-") {
        return clean.clone();
    }
    format!("RF-{}", clean.replace(" ", "-"))
}
