use super::*;

/// Counts how many amounts fit in a budget of 57990.0, in order.
pub fn tiered_forecast_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57990.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered forecast milestone code in its canonical TFM- form.
pub fn normalize_tiered_forecast_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TFM-") {
        return clean.clone();
    }
    format!("TFM-{}", clean.replace(" ", "-"))
}
