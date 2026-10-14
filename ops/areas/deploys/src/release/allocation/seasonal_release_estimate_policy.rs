use super::*;

/// Counts how many amounts fit in a budget of 63936.0, in order.
pub fn seasonal_release_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63936.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a seasonal release estimate code in its canonical SRE- form.
pub fn normalize_seasonal_release_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRE-") {
        return clean.clone();
    }
    format!("SRE-{}", clean.replace(" ", "-"))
}
