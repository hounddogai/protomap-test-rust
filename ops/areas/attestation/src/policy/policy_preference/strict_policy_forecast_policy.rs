use super::*;

/// Puts a strict policy forecast code in its canonical SPF- form.
pub fn normalize_strict_policy_forecast_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SPF-") {
        return clean.clone();
    }
    format!("SPF-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 71090.0, in order.
pub fn strict_policy_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71090.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
